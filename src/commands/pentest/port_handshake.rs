use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;


// 识别MySQL
pub fn is_mysql_handshake(buf: &[u8]) -> bool {
    buf.len() > 5 && buf[4] == 0x0A
}

pub fn extract_mysql_banner(buf: &[u8]) -> String {
    let mut banner = String::new();

    // 提取版本号（从第5字节开始到第一个0结束）
    if let Some(end) = buf.get(5..).and_then(|s| s.iter().position(|&b| b == 0)) {
        let version = &buf[5..5 + end];
        if let Ok(version_str) = std::str::from_utf8(version) {
            banner.push_str("MySQL ");
            banner.push_str(version_str);
        }
    }

    // 提取认证插件名（通常在末尾，遇到连续字符串“caching_sha2_password”）
    if let Some(start) = buf.windows(4).position(|w| w == [0x00, 0x4D, 0x59, 0x53]) {
        if let Ok(plugin) = std::str::from_utf8(&buf[start + 1..]) {
            banner.push_str(" ");
            banner.push_str(plugin);
        }
    } else if let Some(pos) = buf.iter().rposition(|&b| b == 0) {
        if pos + 1 < buf.len() {
            if let Ok(plugin_str) = std::str::from_utf8(&buf[pos + 1..]) {
                if !plugin_str.trim().is_empty() {
                    banner.push_str(" ");
                    banner.push_str(plugin_str.trim());
                }
            }
        }
    }

    banner
}

// 识别RDP服务
pub fn is_rdp_response(buf: &[u8]) -> bool {
    buf.len() >= 7 && buf[0] == 0x03 && buf[1] == 0x00 && buf[4] == 0x02 && buf[5] == 0xF0 && buf[6] == 0x80
}

pub fn extract_rdp_banner(buf: &[u8]) -> String {
    if is_rdp_response(buf) {
        "RDP Protocol Detected".to_string()
    } else {
        "Unknown RDP Response".to_string()
    }
}

// 识别rdp协议，暂存
pub async fn send_rdp_probe(stream: &mut TcpStream) -> Option<String> {
    const RDP_NEGOTIATION_REQUEST: &[u8] = &[
        0x03, 0x00, 0x00, 0x13,
        0x0e, 0xe0, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01,
        0x00, 0x08, 0x00, 0x03,
        0x00, 0x00, 0x00, 0x00,
    ];

    // 1. 发送 Negotiation Request
    if stream.write_all(RDP_NEGOTIATION_REQUEST).await.is_err() {
        return None;
    }

    let mut buf = [0u8; 512];
    let n = match stream.read(&mut buf).await {
        Ok(n) => n,
        Err(_) => return None,
    };

    // 2. 判断是否为 RDP 响应（X.224 Connection Confirm）
    // 第 6 个字节为 0xD0 表示确认包（Connection Confirm）
    if n >= 11 && buf[5] == 0xD0 {
        // ✅ 是 RDP 响应，立即返回
        return Some("RDP".to_string());
    }

    None
}

pub async fn try_http_probe(
    stream: &mut TcpStream,
    ip: &str,
    buf: &mut [u8],
) -> Option<String> {
    let request = format!("GET / HTTP/1.0\r\nHost: {}\r\n\r\n", ip);

    if stream.write_all(request.as_bytes()).await.is_ok() {
        buf.fill(0);
        if let Ok(Ok(n)) = tokio::time::timeout(Duration::from_secs(2), stream.read(buf)).await {
            if n > 0 {
                return Some(extract_banner_text(&buf[..n]));
            }
        }
    }

    None
}

pub fn extract_banner_text(buf: &[u8]) -> String {
    if is_mysql_handshake(buf) {
        return extract_mysql_banner(buf);
    } else if is_rdp_response(buf) {
        return extract_rdp_banner(buf);
    }

    // HTTP(S) 检测
    if let Ok(text) = std::str::from_utf8(buf) {
        if text.starts_with("HTTP/") {
            let mut status_line = None;
            let mut server_line = None;

            for line in text.lines() {
                if status_line.is_none() && line.starts_with("HTTP/") {
                    status_line = Some(line.trim());
                } else if line.to_ascii_lowercase().starts_with("server:") {
                    server_line = Some(line.trim());
                }

                if status_line.is_some() && server_line.is_some() {
                    break;
                }
            }

            return match (status_line, server_line) {
                (Some(status), Some(server)) => format!("{} | {}", status, server),
                (Some(status), None) => status.to_string(),
                _ => "HTTP Response".to_string(),
            };
        }
    }
    // fallback：尝试直接解码为字符串
    match std::str::from_utf8(buf) {
        Ok(s) => s.trim().to_string(),
        Err(_) => buf
            .iter()
            .filter(|&&b| b.is_ascii_graphic() || b == b' ')
            .map(|&b| b as char)
            .collect::<String>(),
    }
}
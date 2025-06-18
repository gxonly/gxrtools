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
    println!("{:?}",buf);
    println!("hello");
    buf.len() >= 7 && buf[0] == 0x03 && buf[1] == 0x00 && buf[4] == 0x02 && buf[5] == 0xF0 && buf[6] == 0x80
}

pub fn extract_rdp_banner(buf: &[u8]) -> String {
    if is_rdp_response(buf) {
        "RDP Protocol Detected".to_string()
    } else {
        "Unknown RDP Response".to_string()
    }
}

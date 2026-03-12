//! 路由追踪（纯 Rust 实现，不调用系统 traceroute）
//!
//! 通过发送 UDP 探测包并逐跳递增 TTL，接收中间路由器返回的 ICMP Time Exceeded 或
//! 目标返回的 ICMP Dest Unreachable，解析 IP/ICMP 包并打印每一跳。
//! 注意：接收 ICMP 需要 raw socket，Linux/macOS 通常需 root，Windows 需管理员。

use clap::Parser;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::mem::MaybeUninit;
use std::time::{Duration, Instant};
use std::{error::Error, thread};

const RECV_BUF_LEN: usize = 1500;
const BASE_PORT: u16 = 33434;
/// IP 首部最小长度（字节）
const IP_HDR_MIN_LEN: usize = 20;
/// ICMP 首部最小长度（类型 + 代码 + 校验和）
const ICMP_HDR_LEN: usize = 4;
/// ICMP Destination Unreachable
const ICMP_TYPE_DEST_UNREACHABLE: u8 = 3;
/// ICMP code: Port unreachable（说明已到达目标主机）
const ICMP_CODE_PORT_UNREACHABLE: u8 = 3;

#[derive(Parser, Debug)]
#[command(name = "trace", about = "路由追踪（纯 Rust 实现，不调用系统 traceroute）")]
pub struct TraceArgs {
    /// 目标主机（IP 或域名）
    #[arg(short, long, value_name = "TARGET")]
    pub target: String,

    /// 最大跳数
    #[arg(short = 'm', long, default_value = "30", value_name = "N")]
    pub max_hops: u8,

    /// 每跳超时（秒）
    #[arg(short = 'T', long, default_value = "3", value_name = "SECS")]
    pub timeout: u64,

    /// 每跳探测次数（用于统计 RTT）
    #[arg(short = 'q', long, default_value = "3", value_name = "N")]
    pub nqueries: u32,
}

/// 解析收到的 IP 包，返回 IP 首部长度（字节）和 ICMP 载荷起始偏移
#[inline]
fn ip_header_len(packet: &[u8]) -> Option<usize> {
    if packet.len() < IP_HDR_MIN_LEN {
        return None;
    }
    let ihl = (packet[0] & 0x0F) as usize;
    if ihl < 5 {
        return None;
    }
    Some(ihl * 4)
}

/// 从 ICMP 载荷中读取 type/code（ICMP 头前 2 字节）
#[inline]
fn icmp_type_code(packet: &[u8], icmp_start: usize) -> Option<(u8, u8)> {
    if packet.len() < icmp_start + ICMP_HDR_LEN {
        return None;
    }
    Some((packet[icmp_start], packet[icmp_start + 1]))
}

pub fn run(args: &TraceArgs) -> Result<(), Box<dyn Error + Send + Sync>> {
    let dest_ip = resolve_target(&args.target)?;
    let timeout = Duration::from_secs(args.timeout);

    println!(
        "🚀 路由追踪到 {} ({})，最多 {} 跳\n",
        args.target, dest_ip, args.max_hops
    );

    // 接收 ICMP 的 raw socket（仅 IPv4）
    let icmp_sock = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))
        .map_err(|e| format!("创建 ICMP raw socket 失败（可能需要 root/管理员）: {}", e))?;
    icmp_sock.set_read_timeout(Some(timeout))?;
    icmp_sock.set_nonblocking(false)?;
    icmp_sock.bind(&SockAddr::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        0,
    )))?;

    let mut reached = false;
    for ttl in 1..=args.max_hops {
        let mut rtts: Vec<Duration> = Vec::with_capacity(args.nqueries as usize);
        let mut reply_ip: Option<IpAddr> = None;

        for _ in 0..args.nqueries {
            let udp_sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
            udp_sock.set_ttl(ttl as u32)?;
            udp_sock.set_write_timeout(Some(Duration::from_secs(1)))?;
            udp_sock.bind(&SockAddr::from(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                0,
            )))?;
            let udp_std: UdpSocket = udp_sock.into();

            let dest = SocketAddr::new(dest_ip, BASE_PORT + ttl as u16);
            let send_time = Instant::now();
            let _ = udp_std.send_to(&[0u8; 0], dest);

            let mut buf = [MaybeUninit::<u8>::uninit(); RECV_BUF_LEN];
            match icmp_sock.recv_from(&mut buf) {
                Ok((n, addr)) => {
                    let elapsed = send_time.elapsed();
                    let src_ip = addr
                        .as_socket()
                        .map(|s| s.ip())
                        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
                    reply_ip.get_or_insert(src_ip);
                    rtts.push(elapsed);

                    let packet: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };
                    let ip_len = match ip_header_len(packet) {
                        Some(l) if packet.len() >= l + ICMP_HDR_LEN => l,
                        _ => continue,
                    };
                    let (icmp_type, icmp_code) =
                        match icmp_type_code(packet, ip_len) {
                            Some(tc) => tc,
                            None => continue,
                        };

                    if icmp_type == ICMP_TYPE_DEST_UNREACHABLE
                        && icmp_code == ICMP_CODE_PORT_UNREACHABLE
                    {
                        reached = true;
                    }
                    // 同一跳后续 probe 只收包计 RTT，不再重复解析
                }
                Err(_) => {
                    // 超时或错误，本 probe 无响应
                }
            }
        }

        // 输出本跳
        let (label, rtt_str) = if let Some(ip) = reply_ip {
            let rtt_str = if rtts.is_empty() {
                "".to_string()
            } else {
                let sum: u128 = rtts.iter().map(|d| d.as_millis()).sum();
                let avg = sum / rtts.len() as u128;
                format!("  {:.0} ms", avg)
            };
            if reached {
                (format!("{}  🎯 到达目标", ip), rtt_str)
            } else {
                (format!("{}  ⏱ TTL 超时", ip), rtt_str)
            }
        } else {
            ("*  请求超时".to_string(), "".to_string())
        };

        println!("{:>3}  {} {}", ttl, label, rtt_str);

        if reached {
            println!("\n✅ 追踪完成");
            return Ok(());
        }

        thread::sleep(Duration::from_millis(100));
    }

    println!("\n❌ 未在 {} 跳内到达目标", args.max_hops);
    Ok(())
}

/// 将目标（域名或 IP 字符串）解析为 IPv4 地址
fn resolve_target(target: &str) -> Result<IpAddr, Box<dyn Error + Send + Sync>> {
    // 若已是 IP，直接解析
    if let Ok(ip) = target.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(v4) => return Ok(IpAddr::V4(v4)),
            IpAddr::V6(_) => return Err("当前仅支持 IPv4，请使用 IPv4 地址".into()),
        }
    }

    // 按域名解析，取第一个 IPv4
    let addrs = (target, 0u16)
        .to_socket_addrs()
        .map_err(|e| format!("解析目标 {} 失败: {}", target, e))?;
    for addr in addrs {
        if let IpAddr::V4(v4) = addr.ip() {
            return Ok(IpAddr::V4(v4));
        }
    }
    Err(format!("无法将 {} 解析为 IPv4 地址", target).into())
}

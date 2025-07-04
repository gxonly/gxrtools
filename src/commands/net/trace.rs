use clap::Parser;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use std::{error::Error, thread};
use std::mem::MaybeUninit;

#[derive(Parser, Debug)]
#[command(name = "gxrtrace", version, about = "A simple cross-platform traceroute tool")]
pub struct TraceArgs {
    /// Target IP or domain
    #[arg(short, long)]
    pub target: String,

    /// Maximum number of hops
    #[arg(short = 'm', long, default_value = "30")]
    pub max_hops: u8,

    /// Timeout per hop (seconds)
    #[arg(short = 'T', long, default_value = "3")]
    pub timeout: u64,
}

pub fn run(args: &TraceArgs) -> Result<(), Box<dyn Error>> {
    let destination: IpAddr = args.target.parse()?;
    let timeout = Duration::from_secs(args.timeout);
    let base_port = 33434;

    println!("🚀 Tracing route to {} (max {} hops):\n", destination, args.max_hops);

    // 创建用于接收 ICMP 的 RAW socket
    let icmp_sock = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
    icmp_sock.set_read_timeout(Some(timeout))?;
    icmp_sock.bind(&SockAddr::from(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)))?;

    for ttl in 1..=args.max_hops {
        let udp_sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        udp_sock.set_ttl(ttl as u32)?;
        udp_sock.bind(&SockAddr::from(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0)))?;
        let udp_std: UdpSocket = udp_sock.into();

        let target = SocketAddr::new(destination, base_port + ttl as u16);
        let send_time = Instant::now();
        let _ = udp_std.send_to(&[0], target); // 发送 UDP 探测包

        let mut buf = [MaybeUninit::<u8>::uninit(); 1500];
        match icmp_sock.recv_from(&mut buf) {
            Ok((n, addr)) => {
                let elapsed = send_time.elapsed();
                let resp_ip = addr.as_socket().map(|s| s.ip()).unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
                let packet = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };

                // 跳过 IP 首部（通常为 20 字节），定位到 ICMP 头
                let icmp_packet = if packet.len() >= 20 {
                    &packet[20..]
                } else {
                    println!("{:>3}: {} ⚠ ICMP packet too short", ttl, resp_ip);
                    continue;
                };

                if icmp_packet.len() >= 2 {
                    let icmp_type = icmp_packet[0];
                    let icmp_code = icmp_packet[1];

                    match icmp_type {
                        11 => {
                            println!("{:>3}: {} ⏱️ TTL exceeded ({:.2?})", ttl, resp_ip, elapsed);
                        }
                        3 => {
                            if icmp_code == 3 {
                                println!("{:>3}: {} 🎯 Target reached! ({:.2?})", ttl, resp_ip, elapsed);
                                return Ok(());
                            } else {
                                println!("{:>3}: {} ⚠ Unreachable (code {})", ttl, resp_ip, icmp_code);
                            }
                        }
                        _ => {
                            println!("{:>3}: {} ❓ Unknown ICMP type/code: {}/{}", ttl, resp_ip, icmp_type, icmp_code);
                        }
                    }
                } else {
                    println!("{:>3}: {} ⚠ ICMP payload too short", ttl, resp_ip);
                }

            }
            Err(_) => {
                println!("{:>3}: 请求超时", ttl);
            }
        }

        thread::sleep(Duration::from_millis(300));
    }

    println!("\n❌ 未能到达目标 {}", args.target);
    Ok(())
}


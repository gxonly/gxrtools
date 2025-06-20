use crate::commands::tools::port_handshake::*;
use crate::commands::tools::port_list::*;
use crate::utils::{parse_ports, parse_targets, save_to_excel};
use clap::Parser;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Semaphore};


#[derive(Parser, Debug)]
pub struct PortScan {
    /// IP 或 IP 段（支持CIDR、范围、多个IP用逗号隔开）
    #[arg(short, long)]
    pub targets: String,

    /// 自定义端口（用逗号隔开，例如：80,443,22）
    #[arg(short, long)]
    pub ports: Option<String>,

    /// 是否扫描全部端口（1-65535）
    #[arg(long, default_value = "false")]
    pub full: bool,


    /// 最大并发数
    #[arg(short = 'c', long, default_value = "1000")]
    pub concurrency: usize,

    /// 输出到excel
    #[arg(long, default_value = "false")]
    pub output: bool,
}





#[derive(Debug, Clone)]
pub struct PortScanResult {
    pub ip: String,
    pub port: u16,
    pub status: String,
    pub banner: String,
}



// 统一的 banner 清洗函数


pub async fn run(args: &PortScan) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start = Instant::now();
    let ips = parse_targets(&args.targets)?;
    let ports: Vec<u16> = if args.full {
        (1..=65535).collect()
    } else if let Some(pstr) = &args.ports {
        parse_ports(pstr)
    } else {
        DEFAULT_PORTS.to_vec()
    };
    println!("🔍 共 {} 个IP，{} 个端口待扫描", ips.len(), ports.len());

    let sem = Arc::new(Semaphore::new(args.concurrency));
    let result_arc = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();
    for ip in ips {
        for &port in &ports {
            let permit = sem.clone().acquire_owned().await?;
            let ip = ip.clone();
            let result_arc = result_arc.clone();
            let handle = tokio::spawn(async move {
                let socket = format!("{}:{}", ip, port);
                let addr: SocketAddr = match socket.parse() {
                    Ok(a) => a,
                    Err(_) => return,
                };

                let (status, banner) =
                    match tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(addr))
                        .await
                    {
                        Ok(Ok(mut stream)) => {
                            let mut buf = vec![0; 1024];
                            let mut banner = String::new();

                            // 第一次尝试直接读取（被动 banner）
                            if let Ok(n) = tokio::time::timeout(Duration::from_secs(1), stream.read(&mut buf)).await
                            {
                                if let Ok(n) = n {
                                    if n > 0 {
                                        banner = extract_banner_text(&buf[..n]);
                                    }
                                }
                            }

                            // 匹配默认字典
                            if banner.trim().is_empty() {
                                if let Some(service) = DEFAULT_PORT_BANNERS.get(&port) {
                                        banner = service.to_string();
                                    }

                            }


                            // Step 3: HTTP 探测（作为兜底）
                            if banner.trim().is_empty() {
                                if let Some(http_banner) = try_http_probe(&mut stream, &ip, &mut buf).await {
                                    banner = http_banner;
                                } else if let Ok(Ok(mut retry_stream)) =
                                    tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(addr)).await
                                {
                                    if let Some(http_banner) = try_http_probe(&mut retry_stream, &ip, &mut buf).await {
                                        banner = http_banner;
                                    }
                                }
                            }


                            ("开放", banner)
                        }
                        _ => ("关闭", String::new()),
                    };

                if status == "开放" {
                    println!("{} => {:<5} | 开放 | 服务: {}", ip, port, banner);
                }

                let mut result = result_arc.lock().await;
                result.push(PortScanResult {
                    ip,
                    port,
                    status: status.to_string(),
                    banner,
                });

                drop(permit);
            });

            handles.push(handle);
        }
    }

    for h in handles {
        let _ = h.await;
    }

    let results = Arc::try_unwrap(result_arc).unwrap().into_inner();

    // 输出到 Excel
    if args.output {
        save_to_excel(
            &results,
            &["IP地址", "端口", "状态", "服务"],
            |r| {
                vec![
                    r.ip.clone(),
                    r.port.to_string(),
                    r.status.clone(),
                    r.banner.clone(),
                ]
            },
            "portscan",
            "portscan",
        )?;
    }
    let elapsed = start.elapsed();
    println!(
        "✅ 扫描完成，共发现 {} 个开放端口",
        results.iter().filter(|r| r.status == "开放").count()
    );
    println!("⏱️ 总耗时：{elapsed:.2?}");

    Ok(())
}

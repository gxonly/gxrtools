use clap::Parser;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::Semaphore;
use rust_xlsxwriter::{Workbook, Format};
use chrono::Local;
use std::error::Error;
use crate::utils::ensure_output_dir;


#[derive(Parser, Debug)]
pub struct PingArgs {
    /// IP地址或网段（CIDR），如：192.168.1.1 或 192.168.1.0/24
    #[arg(short, long)]
    pub target: String,

    /// 超时时间（秒）
    #[arg(short = 'T', long, default_value = "2")]
    pub timeout: u64,

    /// 最大并发数
    #[arg(short = 'c', long, default_value = "100")]
    pub concurrency: usize,

    /// 是否打印结果到终端
    #[arg(short = 'e', long)]
    pub echo: bool,
}

#[derive(Debug)]
pub struct PingResult {
    pub ip: String,
    pub status: String,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = PingArgs::parse();
    run(&args).await
}

pub async fn run(args: &PingArgs) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start = Instant::now();
    let results: Vec<PingResult>;
    let ip_list = parse_targets(&args.target)?;
    println!("🔍 共解析出 {} 个目标 IP", ip_list.len());

    // if args.target.contains('/') {
    //     match generate_ips_from_cidr(&args.target) {
    //         Ok(ips) => {
    //             println!("🔍 正在并发 ping 网段：{}（{}个IP）", args.target, ips.len());
    //             results = ping_concurrent_async(ips, args.timeout, args.concurrency).await?;
    //         }
    //         Err(e) => eprintln!("❌ 无效的网段格式: {}", e),
    //     }
    // } else {
    //     println!("🔍 正在 ping 单个 IP: {}", args.target);
    //     let result = ping_ip_async(&args.target, args.timeout).await?;
    //     results.push(result);
    // }
    results = ping_concurrent_async(ip_list, args.timeout, args.concurrency).await?;
    if args.echo {
        println!("\n📋 扫描结果：");
        for r in &results {
            println!("{} => {}", r.ip, r.status);
        }
    }

    save_to_excel(&results)?;

    let elapsed = start.elapsed();
    println!("⏱️ 总耗时：{elapsed:.2?}");

    Ok(())
}

fn parse_targets(targets: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let mut all_ips = Vec::new();
    for target in targets.split(',') {
        let target = target.trim();
        if target.contains('/') {
            // CIDR 格式
            let cidr_ips = generate_ips_from_cidr(target)?;
            all_ips.extend(cidr_ips);
        } else if let Some(_) = target.rfind('-') {
            // IP 范围格式：192.168.1.5-10
            let range_ips = generate_ips_from_range(target)?;
            all_ips.extend(range_ips);
        } else {
            // 单个 IP
            let ip = Ipv4Addr::from_str(target)
                .map_err(|_| format!("无效的 IP 地址: {}", target))?;
            all_ips.push(ip.to_string());
        }
    }

    Ok(all_ips)
}


fn generate_ips_from_range(range_str: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let dash_pos = range_str.rfind('-').ok_or("无效的 IP 范围格式")?;
    let (base, end) = range_str.split_at(dash_pos);
    let base_ip = Ipv4Addr::from_str(base).map_err(|_| "无效的 IP 地址")?;
    let end_part = &end[1..]; // 去掉 '-'

    let base_parts: Vec<u8> = base_ip.octets().to_vec();
    let end_last = end_part.parse::<u8>().map_err(|_| "IP 范围结束值无效")?;

    if end_last < base_parts[3] {
        return Err("IP 范围结束值必须大于开始值".into());
    }

    let mut ips = Vec::new();
    for i in base_parts[3]..=end_last {
        let ip = Ipv4Addr::new(base_parts[0], base_parts[1], base_parts[2], i);
        ips.push(ip.to_string());
    }

    Ok(ips)
}



async fn ping_ip_async(ip: &str, timeout_secs: u64) -> Result<PingResult, Box<dyn Error + Send + Sync>> {
    let timeout_str = format!("{}", timeout_secs * 1000);
    let ip_str = ip.to_string();

    let output = if cfg!(target_os = "windows") {
        Command::new("ping")
            .args(["-n", "1", "-w", &timeout_str, &ip_str])
            .output()
            .await
    } else {
        Command::new("ping")
            .args(["-c", "1", "-W", &timeout_secs.to_string(), &ip_str])
            .output()
            .await
    };

    let status = match output {
        Ok(out) if out.status.success() => "成功",
        Ok(_) => "失败",
        Err(_) => "错误",
    };

    Ok(PingResult {
        ip: ip.to_string(),
        status: status.to_string(),
    })
}

async fn ping_concurrent_async(
    ips: Vec<String>,
    timeout: u64,
    concurrency: usize,
) -> Result<Vec<PingResult>, Box<dyn Error + Send + Sync>> {
    let sem = Arc::new(Semaphore::new(concurrency));
    let result_arc = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for ip in ips {
        let permit = sem.clone().acquire_owned().await?;
        let ip_clone = ip.clone();
        let result_clone = Arc::clone(&result_arc);

        let handle = tokio::spawn(async move {
            let r = ping_ip_async(&ip_clone, timeout).await;
            if let Ok(res) = r {
                let mut vec = result_clone.lock().await;
                vec.push(res);
            }
            drop(permit);
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let final_result = Arc::try_unwrap(result_arc)
        .unwrap()
        .into_inner();

    Ok(final_result)
}



fn save_to_excel(results: &[PingResult]) -> Result<String, Box<dyn Error + Send + Sync>> {
    let output_dir = ensure_output_dir("output/ping")?;

    let timestamp = Local::now().format("%Y%m%d%H%M").to_string();
    let filename = format!("{}_ping.xlsx", timestamp);
    let filepath = output_dir.join(&filename);

    let mut workbook = Workbook::new(filepath.to_str().unwrap());
    let worksheet = workbook.add_worksheet();
    let default_format = Format::default();

    worksheet.write_string(0, 0, "IP地址", &default_format)?;
    worksheet.write_string(0, 1, "状态", &default_format)?;

    for (i, result) in results.iter().enumerate() {
        worksheet.write_string((i + 1) as u32, 0, &result.ip, &default_format)?;
        worksheet.write_string((i + 1) as u32, 1, &result.status, &default_format)?;
    }

    workbook.close()?;
    println!("✅ 结果已存储至：output/{}", &filename);
    Ok(filepath.to_string_lossy().to_string())
}

fn generate_ips_from_cidr(cidr: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return Err("CIDR 格式错误".into());
    }

    let base_ip = Ipv4Addr::from_str(parts[0]).map_err(|_| "无效的IP地址".to_string())?;
    let subnet_mask = parts[1].parse::<u8>().map_err(|_| "无效的子网掩码".to_string())?;

    if subnet_mask > 32 {
        return Err("子网掩码不能大于32".into());
    }

    let ip_u32 = u32::from(base_ip);
    let num_ips = 2u32.pow((32 - subnet_mask) as u32);
    let start = ip_u32 + 1;
    let end = ip_u32 + num_ips - 2;

    let mut ips = Vec::new();
    for i in start..=end {
        ips.push(Ipv4Addr::from(i).to_string());
    }

    Ok(ips)
}

use clap::Parser;
use crate::utils::{parse_targets, save_to_excel};
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::Semaphore;

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
    results = ping_concurrent_async(ip_list, args.timeout, args.concurrency).await?;
    if args.echo {
        println!("\n📋 扫描结果：");
        for r in &results {
            println!("{} => {}", r.ip, r.status);
        }
    }

    // 输出到ping的excel中
    save_to_excel(
        &results,
        &["IP地址", "状态"],
        |item| vec![item.ip.clone(), item.status.clone()],
        "ping",
        "ping",
    )?;
    let elapsed = start.elapsed();
    println!("⏱️ 总耗时：{elapsed:.2?}");

    Ok(())
}

// 整理IP地址为列表

async fn ping_ip_async(
    ip: &str,
    timeout_secs: u64,
) -> Result<PingResult, Box<dyn Error + Send + Sync>> {
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

    let final_result = Arc::try_unwrap(result_arc).unwrap().into_inner();

    Ok(final_result)
}

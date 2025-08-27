use crate::utils::{ScanProgress, parse_targets, save_to_excel};
use clap::Parser;
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

    /// 输出到excel
    #[arg(long, default_value = "false")]
    pub output: bool,

    /// 每个IP最多ping的次数
    #[arg(short, long, default_value = "3")]
    pub count: u32,
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
    println!("🔍 共测试 {} 个 IP", ip_list.len());
    let mut progress = ScanProgress::new(ip_list.len() as u64);
    results = ping_concurrent_async(ip_list, args.timeout, args.count, args.concurrency, &mut progress).await?;
    let mut success_count = 0;

    progress.println("📋 扫描结果：".to_string());
    for r in &results {
        if r.status == "成功" {
            if args.echo {
                progress.println(format!("{} => {}", r.ip, r.status));
            }
            success_count += 1;
        }
    }

    // 输出到ping的excel中
    if args.output {
        save_to_excel(
            &results,
            &["IP地址", "状态"],
            |item| vec![item.ip.clone(), item.status.clone()],
            "ping",
            "ping",
        )?;
    }
    let elapsed = start.elapsed();
    progress.finish();
    println!("⏱️ 总耗时：{elapsed:.2?}，共识别存活主机{success_count}个。");
    Ok(())
}

// ping 单个 IP，最多尝试 count 次，只要一次成功即成功
async fn ping_ip_async(
    ip: &str,
    timeout_secs: u64,
    count: u32,
) -> Result<PingResult, Box<dyn Error + Send + Sync>> {
    let timeout_str = format!("{}", timeout_secs * 1000);
    let ip_str = ip.to_string();
    let mut success = false;

    for _ in 0..count {
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

        if let Ok(out) = output {
            if out.status.success() {
                success = true;
                break; // 一次成功就终止
            }
        }
    }

    let status = if success { "成功" } else { "失败" };

    Ok(PingResult {
        ip: ip.to_string(),
        status: status.to_string(),
    })
}

pub async fn ping_concurrent_async(
    ips: Vec<String>,
    timeout: u64,
    count: u32,
    concurrency: usize,
    progress: &mut ScanProgress,
) -> Result<Vec<PingResult>, Box<dyn Error + Send + Sync>> {
    let sem = Arc::new(Semaphore::new(concurrency));
    let result_arc = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for ip in ips {
        let progress = progress.clone();
        let permit = sem.clone().acquire_owned().await?;
        let ip_clone = ip.clone();
        let result_clone = Arc::clone(&result_arc);

        let handle = tokio::spawn(async move {
            let r = ping_ip_async(&ip_clone, timeout, count).await;
            if let Ok(res) = r {
                let mut vec = result_clone.lock().await;
                vec.push(res);
                progress.inc();
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
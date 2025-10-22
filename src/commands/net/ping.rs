// src/commands/net/ping.rs
use crate::utils::{ScanProgress, parse_targets, save_to_excel};
use clap::Parser;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::Semaphore;

/// Ping扫描参数配置
#[derive(Parser, Debug)]
pub struct PingArgs {
    /// IP地址或网段（支持CIDR、范围、多个IP用逗号隔开）
    ///
    /// 示例：
    /// - 单个IP: 192.168.1.1
    /// - 多个IP: 192.168.1.1,192.168.1.2
    /// - IP范围: 192.168.1.1-10
    /// - CIDR: 192.168.1.0/24
    #[arg(short, long, value_name = "TARGET")]
    pub target: String,

    /// 超时时间（秒）
    #[arg(short = 'T', long, default_value = "2", value_name = "SECS")]
    pub timeout: u64,

    /// 最大并发数
    #[arg(short = 'c', long, default_value = "100", value_name = "NUM")]
    pub concurrency: usize,

    /// 每个IP的ping次数（只要有一次成功即判定为存活）
    #[arg(short = 'n', long, default_value = "3", value_name = "COUNT")]
    pub count: u32,

    /// 是否打印详细结果到终端
    #[arg(short = 'e', long)]
    pub echo: bool,

    /// 是否输出结果到Excel文件
    #[arg(short = 'o', long)]
    pub output: bool,
}

/// Ping扫描结果
#[derive(Debug, Clone)]
pub struct PingResult {
    /// IP地址
    pub ip: String,
    /// 状态（成功/失败）
    pub status: String,
    /// 响应时间（毫秒，可选）
    pub response_time: Option<f64>,
}

impl PingResult {
    /// 创建成功的ping结果
    fn success(ip: String, response_time: Option<f64>) -> Self {
        Self {
            ip,
            status: "成功".to_string(),
            response_time,
        }
    }

    /// 创建失败的ping结果
    fn failure(ip: String) -> Self {
        Self {
            ip,
            status: "失败".to_string(),
            response_time: None,
        }
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.status == "成功"
    }
}

/// 执行Ping扫描
///
/// # 参数
/// * `args` - Ping扫描参数
///
/// # 返回
/// * `Ok(())` - 扫描成功完成
/// * `Err` - 扫描过程中发生错误
pub async fn run(args: &PingArgs) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start = Instant::now();

    // 解析目标IP列表
    let ip_list = parse_targets(&args.target)?;
    let total_ips = ip_list.len();

    if total_ips == 0 {
        return Err("未解析到任何有效的IP地址".into());
    }

    println!("🔍 开始Ping扫描，共 {} 个目标IP", total_ips);
    println!(
        "⚙️  配置: 超时={}秒, 重试={}次, 并发={}",
        args.timeout, args.count, args.concurrency
    );

    // 创建进度条
    let progress = ScanProgress::new(total_ips as u64);

    // 执行并发ping扫描
    let results = ping_concurrent_async(
        ip_list,
        args.timeout,
        args.count,
        args.concurrency,
        &progress,
    )
    .await?;

    // 统计结果
    let success_count = results.iter().filter(|r| r.is_success()).count();
    let failure_count = total_ips - success_count;

    // 打印详细结果
    if args.echo {
        progress.println("📋 扫描结果：".to_string());
        for result in &results {
            if result.is_success() {
                let time_info = result
                    .response_time
                    .map(|t| format!(" ({}ms)", t))
                    .unwrap_or_default();
                progress.println(format!("  ✅ {} => 存活{}", result.ip, time_info));
            }
        }
    }

    progress.finish_with_message("✅ Ping扫描完成");

    // 保存到Excel
    if args.output {
        save_to_excel(
            &results,
            &["IP地址", "状态", "响应时间(ms)"],
            |item| {
                vec![
                    item.ip.clone(),
                    item.status.clone(),
                    item.response_time
                        .map(|t| format!("{:.2}", t))
                        .unwrap_or_else(|| "-".to_string()),
                ]
            },
            "ping",
            "ping",
        )?;
    }

    // 打印总结
    let elapsed = start.elapsed();
    println!("\n📊 扫描统计:");
    println!("   总计: {} 个IP", total_ips);
    println!(
        "   存活: {} 个 ({:.1}%)",
        success_count,
        (success_count as f64 / total_ips as f64) * 100.0
    );
    println!(
        "   失败: {} 个 ({:.1}%)",
        failure_count,
        (failure_count as f64 / total_ips as f64) * 100.0
    );
    println!("   耗时: {:.2?}", elapsed);

    Ok(())
}

/// 并发执行Ping扫描
///
/// # 参数
/// * `ips` - IP地址列表
/// * `timeout` - 超时时间（秒）
/// * `count` - 每个IP的ping次数
/// * `concurrency` - 最大并发数
/// * `progress` - 进度条
///
/// # 返回
/// * `Ok(Vec<PingResult>)` - Ping结果列表
/// * `Err` - 扫描失败
pub async fn ping_concurrent_async(
    ips: Vec<String>,
    timeout: u64,
    count: u32,
    concurrency: usize,
    progress: &ScanProgress,
) -> Result<Vec<PingResult>, Box<dyn Error + Send + Sync>> {
    let sem = Arc::new(Semaphore::new(concurrency));
    let results = Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(ips.len())));
    let mut handles = Vec::with_capacity(ips.len());

    for ip in ips {
        let permit = sem.clone().acquire_owned().await?;
        let ip_clone = ip.clone();
        let results_clone = Arc::clone(&results);
        let progress_clone = progress.clone();

        let handle = tokio::spawn(async move {
            let result = ping_ip_async(&ip_clone, timeout, count).await;

            // 将结果添加到结果列表
            {
                let mut results_guard = results_clone.lock().await;
                results_guard.push(result);
            }

            progress_clone.inc(1);
            drop(permit);
        });

        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("⚠️  任务执行失败: {}", e);
        }
    }

    let final_results = Arc::try_unwrap(results)
        .expect("无法获取最终结果")
        .into_inner();

    Ok(final_results)
}

/// Ping单个IP地址
///
/// 会尝试ping指定次数，只要有一次成功即返回成功结果
///
/// # 参数
/// * `ip` - IP地址
/// * `timeout_secs` - 超时时间（秒）
/// * `count` - 最多尝试次数
///
/// # 返回
/// * `Ok(PingResult)` - Ping结果
/// * `Err` - Ping失败
async fn ping_ip_async(ip: &str, timeout_secs: u64, count: u32) -> PingResult {
    let timeout_str = format!("{}", timeout_secs * 1000);

    for attempt in 1..=count {
        let output = if cfg!(target_os = "windows") {
            // Windows平台: ping -n 1 -w timeout IP
            Command::new("ping")
                .args(["-n", "1", "-w", &timeout_str, ip])
                .output()
                .await
        } else {
            // Unix/Linux平台: ping -c 1 -W timeout IP
            Command::new("ping")
                .args(["-c", "1", "-W", &timeout_secs.to_string(), ip])
                .output()
                .await
        };

        match output {
            Ok(out) if out.status.success() => {
                // 尝试提取响应时间
                let response_time = extract_response_time(&out.stdout);
                return PingResult::success(ip.to_string(), response_time);
            }
            Ok(_) => {
                // Ping失败，继续重试
                if attempt < count {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
            Err(e) => {
                eprintln!("⚠️  执行ping命令失败 {}: {}", ip, e);
                break;
            }
        }
    }

    PingResult::failure(ip.to_string())
}

/// 从ping输出中提取响应时间
///
/// # 参数
/// * `output` - ping命令的标准输出
///
/// # 返回
/// * `Some(f64)` - 响应时间（毫秒）
/// * `None` - 无法提取响应时间
fn extract_response_time(output: &[u8]) -> Option<f64> {
    let output_str = String::from_utf8_lossy(output);

    // Windows格式: "时间=1ms" 或 "time=1ms"
    // Linux格式: "time=1.23 ms"
    if let Some(time_pos) = output_str
        .find("time=")
        .or_else(|| output_str.find("时间="))
    {
        let time_part = &output_str[time_pos..];

        // 查找数字部分
        if let Some(num_start) = time_part.find(|c: char| c.is_ascii_digit()) {
            let num_part = &time_part[num_start..];

            // 提取数字（包括小数点）
            let num_str: String = num_part
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();

            if let Ok(time) = num_str.parse::<f64>() {
                return Some(time);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_result_creation() {
        let success = PingResult::success("192.168.1.1".to_string(), Some(10.5));
        assert!(success.is_success());
        assert_eq!(success.ip, "192.168.1.1");
        assert_eq!(success.response_time, Some(10.5));

        let failure = PingResult::failure("192.168.1.2".to_string());
        assert!(!failure.is_success());
        assert_eq!(failure.ip, "192.168.1.2");
        assert_eq!(failure.response_time, None);
    }

    #[test]
    fn test_extract_response_time_windows() {
        let output = b"Reply from 192.168.1.1: bytes=32 time=15ms TTL=64";
        let time = extract_response_time(output);
        assert_eq!(time, Some(15.0));
    }

    #[test]
    fn test_extract_response_time_linux() {
        let output = b"64 bytes from 192.168.1.1: icmp_seq=1 ttl=64 time=1.23 ms";
        let time = extract_response_time(output);
        assert_eq!(time, Some(1.23));
    }

    #[test]
    fn test_extract_response_time_chinese() {
        let output = "来自 192.168.1.1 的回复: 字节=32 时间=20ms TTL=64".as_bytes();
        let time = extract_response_time(output);
        assert_eq!(time, Some(20.0));
    }

    #[test]
    fn test_extract_response_time_none() {
        let output = b"Request timeout for icmp_seq 1";
        let time = extract_response_time(output);
        assert_eq!(time, None);
    }
}

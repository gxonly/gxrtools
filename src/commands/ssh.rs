// src/commands/ssh.rs
use async_std::net::TcpStream;
use clap::Parser;
use ssh2::Session;
use calamine::{Reader, Xlsx, open_workbook};
use std::error::Error;
use tokio::task;
use std::fs::{self, File};
use std::io::{Write, Read};  // 添加了Read导入
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(about = "SSH批量命令执行工具", long_about = None)]
pub struct SshArgs {
    /// 远程主机的IP地址 (与 -f 互斥)
    #[arg(short = 'H', long, conflicts_with = "file")]
    pub host: Option<String>,
    
    /// 从Excel文件读取主机列表(格式: 主机,端口,用户名,密码/密钥路径) (与 -H 互斥)
    #[arg(short = 'f', long, conflicts_with = "host")]
    pub file: Option<String>,
    
    /// SSH端口号 (当使用 -H 时有效)
    #[arg(short = 'P', long, default_value = "22", requires = "host")]
    pub port: u16,
    
    /// 用户名 (当使用 -H 时有效)
    #[arg(short = 'u', long, default_value = "root", requires = "host")]
    pub username: String,
    
    /// 密码或私钥路径 (当使用 -H 时必需)
    #[arg(short = 'p', long, requires = "host")]
    pub password_or_key: Option<String>,
    
    /// 要执行的命令
    #[arg(short = 'c', long, required = true)]
    pub command: String,
    
    /// 并发线程数
    #[arg(short = 't', long, default_value = "4")]
    pub threads: usize,

    /// 输出到控制台
    #[arg(short = 'e', long)]
    pub echo: bool,
}

#[derive(Debug, Clone)]
pub struct HostInfo {
    host: String,
    port: u16,
    username: String,
    password_or_key: String,
}

fn ensure_output_dir() -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let output_dir = PathBuf::from("output/ssh");
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }
    Ok(output_dir)
}

fn save_result(host: &str, output: &str, echo: bool) -> Result<(), Box<dyn Error + Send + Sync>> {
    let output_dir = ensure_output_dir()?;
    let filename = format!("{}.txt", host.replace(".", "_"));
    let filepath = output_dir.join(filename);

    let mut file = File::create(filepath)?;
    writeln!(file, "{}", output)?;

    if echo {
        println!("=== 主机 {} 执行结果 ===", host);
        println!("{}", output);
        println!("=====================");
    }

    Ok(())
}

pub async fn run(args: &SshArgs) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 记录开始时间
    let start_time = Instant::now();
    
    // 获取主机列表并同时计算主机数量
    let (hosts, total_hosts) = if let Some(file_path) = &args.file {
        let hosts = read_hosts_from_excel(file_path)?;
        let count = hosts.len();
        (hosts, count)
    } else if let Some(host) = &args.host {
        let hosts = vec![HostInfo {
            host: host.clone(),
            port: args.port,
            username: args.username.clone(),
            password_or_key: args.password_or_key
                .as_ref()
                .ok_or("使用 -H 时必须提供 -p 参数")?
                .clone(),
        }];
        (hosts, 1)
    } else {
        return Err("必须指定 -H (单个主机) 或 -f (主机列表文件)".into());
    };

    ensure_output_dir()?;

    println!("🚀 开始执行SSH批量命令，共 {} 台主机。", total_hosts);
    println!("📋 执行命令: {}", args.command);

    let mut tasks = vec![];
    for host in hosts {
        let cmd = args.command.clone();
        let echo = args.echo;
        tasks.push(task::spawn(async move {
            let result = match connect_ssh(&host.host, host.port, &host.username, &host.password_or_key).await {
                Ok(session) => match execute_command(&session, &cmd).await {
                    Ok((success, output)) => {
                        let status = if success { "✅ 执行成功" } else { "❌ 执行失败" };
                        format!("{}: {}", status, output)
                    }
                    Err(e) => format!("❌ 命令执行失败: {}", e),
                },
                Err(e) => format!("❌ 连接失败: {}", e),
            };

            if let Err(e) = save_result(&host.host, &result, echo) {
                eprintln!("⚠️ 无法保存结果: {}", e);
            }
        }));
    }

    // 等待所有任务完成
    for task in tasks {
        task.await?;
    }

    let duration = start_time.elapsed();
    println!("\n🎉 所有主机执行完成!");
    println!("⏱️ 总耗时: {:.2}秒", duration.as_secs_f64());

    Ok(())
}

async fn connect_ssh(host: &str, port: u16, username: &str, password_or_key: &str) -> Result<Arc<Session>, Box<dyn Error + Send + Sync>> {
    let addr = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(&addr).await?;
    let mut session = Session::new()?;
    
    // 由于ssh2库是同步的，需要使用block_in_place在异步上下文中执行
    task::block_in_place(|| {
        session.set_tcp_stream(tcp);
        session.handshake()?;
        
        if Path::new(password_or_key).exists() {
            session.userauth_pubkey_file(username, None, Path::new(password_or_key), None)?;
        } else {
            session.userauth_password(username, password_or_key)?;
        }
        
        if !session.authenticated() {
            return Err("SSH认证失败".into());
        }
        Ok(Arc::new(session))  // 使用Arc包装Session
    })
}

async fn execute_command(
    session: &Session,
    command: &str,
) -> Result<(bool, String), Box<dyn Error + Send + Sync>> {
    let mut channel = session.channel_session()?;
    channel.exec(command)?;
    
    let mut output = String::new();
    let mut buf = [0u8; 1024];
    
    loop {
        let n = channel.read(&mut buf)?;
        if n == 0 {
            break;
        }
        output.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    
    channel.wait_close()?;
    let exit_status = channel.exit_status()?;
    
    Ok((exit_status == 0, output))
}

fn read_hosts_from_excel<P: AsRef<Path>>(path: P) -> Result<Vec<HostInfo>, Box<dyn Error + Send + Sync>> {
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range("Sheet1")
        .ok_or("找不到工作表 'Sheet1'")??;

    let mut hosts = Vec::new();
    
    for row in range.rows().skip(1) {
        if row.len() < 4 {
            continue;
        }
        
        let host = row[0].to_string();
        let port = row[1].get_float().map(|p| p as u16).unwrap_or(22);
        let username = row[2].to_string();
        let password_or_key = row[3].to_string();
        
        hosts.push(HostInfo {
            host,
            port,
            username,
            password_or_key,
        });
    }

    if hosts.is_empty() {
        return Err("Excel 文件中没有有效的主机数据".into());
    }

    Ok(hosts)
}

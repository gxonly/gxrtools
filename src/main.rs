use clap::{Parser, Subcommand};
use gxtools::commands::{ping, ssh};
// use gxtools::commands::ping;

#[derive(Parser, Debug)]
#[command(name = "myapp")]
#[command(about = "gx工具箱", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 执行 ping 操作
    Ping(ping::PingArgs),
    /// 通过 SSH 执行命令
    Ssh(ssh::SshArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ping(args) => {
            if let Err(e) = ping::run(&args).await {
                eprintln!("Ping错误: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Ssh(args) => {
            if let Err(e) = ssh::run(&args).await {
                eprintln!("SSH执行错误: {}", e);
                std::process::exit(1);
            }
        }
    }
}

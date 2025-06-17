use clap::{Parser, Subcommand};
use gxtools::commands::{mysql, ping, ssh, tools, windows};

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
    /// 执行 Linux 命令（等保基线采集）
    Linux(ssh::SshArgs),
    /// 执行 MySQL 命令（等保基线采集）
    Mysql(mysql::MysqlArgs),
    /// 执行 Windows 命令（等保基线采集）
    Windows(windows::WindowsArgs),
    /// 渗透测试工具
    Tools {
        #[command(subcommand)]
        subcommand: ToolsCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ToolsCommands {
    /// 端口扫描工具
    Portscan(tools::portscan::PortScan),
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
        Commands::Linux(args) => {
            if let Err(e) = ssh::run(&args).await {
                eprintln!("SSH执行错误: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Mysql(args) => {
            if let Err(e) = mysql::run(&args).await {
                eprintln!("Mysql执行错误: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Windows(args) => {
            if let Err(e) = windows::run(&args).await {
                eprintln!("Mysql执行错误: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Tools { subcommand } => {
            match subcommand {
                ToolsCommands::Portscan(args) => {
                    if let Err(e) = tools::portscan::run(&args).await {
                        eprintln!("Portscan执行错误: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

use clap::{Parser, Subcommand};
use gxtools::commands::{net, pentest,check};

#[derive(Parser, Debug)]
#[command(name = "myapp")]
#[command(about = "gx工具箱", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 网络测试模块
    Net {
        #[command(subcommand)]
        subcommand: NetCommands,
    },


    ///等保核查模块
    Check {
        #[command(subcommand)]
        subcommand: CheckCommands,
    },

    /// 渗透测试模块
    Pentest {
        #[command(subcommand)]
        subcommand: PentestCommands,
    },
}

#[derive(Subcommand, Debug)]
enum PentestCommands {
    /// 端口扫描工具
    Portscan(pentest::portscan::PortScan),
}

#[derive(Subcommand, Debug)]
enum NetCommands {
    /// Ping扫描工具
    Ping(net::ping::PingArgs),
}

#[derive(Subcommand, Debug)]
enum CheckCommands {
    /// 执行 Linux 命令（等保基线采集）
    Linux(check::ssh::SshArgs),
    /// 执行 MySQL 命令（等保基线采集）
    Mysql(check::mysql::MysqlArgs),
    /// 执行 Oracle 命令（等保基线采集），待处理兼容新问题
    Oracle(check::oracle::OracleArgs),
    /// 执行 Windows 命令（等保基线采集）
    Windows(check::windows::WindowsArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Net { subcommand } => {
            match subcommand {
                NetCommands::Ping(args) => {
                    if let Err(e) = net::ping::run(&args).await {
                        eprintln!("Ping扫描: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Check { subcommand } => {
            match subcommand {
                CheckCommands::Linux(args) => {
                    if let Err(e) = check::ssh::run(&args).await {
                        eprintln!("SSH执行错误: {}", e);
                        std::process::exit(1);
                    }
                }
                CheckCommands::Mysql(args) => {
                    if let Err(e) = check::mysql::run(&args).await {
                        eprintln!("MySQL执行错误: {}", e);
                        std::process::exit(1);
                    }
                }
                CheckCommands::Oracle(args) => {
                    if let Err(e) = check::oracle::try_set_oracle_client_path() {
                        eprintln!("❌ Oracle 模块初始化失败: {}", e);
                        std::process::exit(1);
                    }
                    if let Err(e) = check::oracle::run(&args).await {
                        eprintln!("Oracle执行错误: {}", e);
                        std::process::exit(1);
                    }
                }
                CheckCommands::Windows(args) => {
                    if let Err(e) = check::windows::run(&args).await {
                        eprintln!("Windows执行错误: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }

        Commands::Pentest { subcommand } => {
            match subcommand {
                PentestCommands::Portscan(args) => {
                    if let Err(e) = pentest::portscan::run(&args).await {
                        eprintln!("Portscan执行错误: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

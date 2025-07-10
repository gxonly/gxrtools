use clap::{Parser, Subcommand};
use gxtools::commands::net::trace;
use gxtools::commands::{check, net, pentest};
use gxtools::commands::pentest::screenshot;
use crate::PentestCommands::Screenshot;

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
    /// poc模块测试
    Poctest(pentest::poctest::PocTest),
    /// URL 路径探测
    Urlscan(pentest::urlscan::UrlScan),
    ///URL截图
    Screenshot(screenshot::ScreenshotArgs),
}

#[derive(Subcommand, Debug)]
enum NetCommands {
    /// Ping扫描工具
    Ping(net::ping::PingArgs),
    /// 路由追踪工具
    Trace(net::trace::TraceArgs),
}

#[derive(Subcommand, Debug)]
enum CheckCommands {
    /// 执行 Linux 命令（等保基线采集）
    Linux(check::ssh::SshArgs),
    /// 执行 MySQL 命令（等保基线采集）
    Mysql(check::mysql::MysqlArgs),
    /// 执行 Oracle 命令（等保基线采集）
    Oracle(check::oracle::OracleArgs),
    /// 执行 Windows 命令（等保基线采集）
    Windows(check::windows::WindowsArgs),
    /// 执行 Redis 命令（等保基线采集）
    Redis(check::redis::RedisArgs),
}

fn handle_error<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) {
    if let Err(e) = result {
        eprintln!("❌ {}: {}", context, e);
        std::process::exit(1);
    }
}


#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Net { subcommand } => match subcommand {
            NetCommands::Ping(args) => {
                handle_error(net::ping::run(&args).await, "Ping 扫描失败");

            }
            NetCommands::Trace(args) => {
                handle_error(trace::run(&args), "Trace 失败");
            }
        },

        Commands::Check { subcommand } => match subcommand {
            CheckCommands::Linux(args) => {
                handle_error(check::ssh::run(&args).await, "SSH执行错误");
            }
            CheckCommands::Mysql(args) => {
                handle_error(check::mysql::run(&args).await, "MySQL执行错误");
            }
            CheckCommands::Oracle(args) => {
                handle_error(check::oracle::try_set_oracle_client_path(), "Oracle 模块初始化失败");
                handle_error(check::oracle::run(&args).await, "Oracle执行错误");
            }
            CheckCommands::Windows(args) => {
                handle_error(check::windows::run(&args).await, "Windows执行错误");
            }
            CheckCommands::Redis(args) => {
                handle_error(check::redis::run(&args).await, "Redis执行错误");
            }
        },

        Commands::Pentest { subcommand } => match subcommand {
            PentestCommands::Portscan(args) => {
                handle_error(pentest::portscan::run(&args).await, "Portscan执行错误");
            }
            PentestCommands::Poctest(args) => {
                handle_error(pentest::poctest::run(&args).await, "Poctest执行错误");
            }
            PentestCommands::Urlscan(args) => {
                handle_error(pentest::urlscan::run(&args).await, "Urlscan错误");
            }
            Screenshot(args) => {
                handle_error(pentest::screenshot::run(&args).await, "Screenshot错误");
            }
        },
    }
}

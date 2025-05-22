use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
pub struct MysqlArgs {
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
    #[arg(short = 'c', long, num_args = 1..)]
    pub commands: Vec<String>,
    
    /// 并发线程数
    #[arg(short = 't', long, default_value = "4")]
    pub threads: usize,

    /// 输出到控制台，使用前提需指定自定义命令
    #[arg(short = 'e', long, requires = "command")]
    pub echo: bool,
}

pub async fn run(_args: &MysqlArgs) -> Result<(), Box<dyn Error + Send + Sync>> {
    Ok(())
}
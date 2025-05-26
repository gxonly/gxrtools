use axum::{
    routing::{get, post},
    extract::{connect_info::ConnectInfo, State},
    response::{IntoResponse, Response},
    Json, Router,
    http::{StatusCode, header},
};
use serde_json::Value;
use std::{net::SocketAddr, sync::Arc};
use encoding_rs::UTF_16LE;
use clap::Args;
use crate::utils::ensure_output_dir;
use hyper::{Server, server::conn::AddrIncoming};
use tokio::fs;
use std::fs as stdfs;
use local_ip_address::local_ip;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Args)]
pub struct WindowsArgs {
    /// 指定ps1脚本路径
    #[arg(short, long)]
    pub file: Option<String>,
    /// 修改端口，默认3000
    #[arg(short, long, default_value = "3000")]
    pub port: u16,
    /// 绑定本机IP，默认自动识别，多网卡可能异常
    #[arg(short, long)]
    pub ip: Option<String>,
}

pub async fn run(args: &WindowsArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let script_path = args.file.clone().unwrap_or_else(|| "windows.ps1".to_string());
    // 获取本机 IP 地址
    let local_ip = match &args.ip {
    Some(ip_str) => IpAddr::from_str(ip_str)?,
    None => local_ip()?,
    };
    println!("当前接收绑定IP地址为：{local_ip}");
    let report_url = format!("http://{}:{}/report", local_ip,&args.port);
    let mut script_content = stdfs::read_to_string(&script_path)
        .unwrap_or_else(|_| String::new());

    let url_line = format!("$url = \"{}\"\n", report_url);

    // 使用简单的正则替换或插入（你也可以使用更精确的处理）
    if script_content.contains("$url = ") {
        script_content = script_content
            .lines()
            .map(|line| {
                if line.trim_start().starts_with("$url = ") {
                    url_line.clone()
                } else {
                    format!("{}\n", line)
                }
            })
            .collect();
    } else {
        // 没有 $url 定义，插入到文件开头
        script_content = format!("{}\n{}", url_line, script_content);
    }
    stdfs::write(&script_path, script_content)?;

    let app = Router::new()
        .route("/script", get(get_script))
        .with_state(Arc::new(script_path))
        .route("/report", post(report_result));

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    println!("Server running at http://{}", addr);

    // 使用 hyper 0.14 的 listener 和 Server
    let listener = AddrIncoming::bind(&addr)?;
    Server::builder(listener)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    Ok(())
}

async fn get_script(State(path): State<Arc<String>>) -> Response {
    match stdfs::read(&**path) {
        Ok(bytes) => {
            let (cow, _, had_errors) = UTF_16LE.decode(&bytes);
            if had_errors {
                eprintln!("编码错误: 无法以 UTF-16LE 解码");
                return (StatusCode::INTERNAL_SERVER_ERROR, "文件编码有误，无法正确解码").into_response();
            }
            let mut res = (StatusCode::OK, cow.into_owned()).into_response();
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            res
        }
        Err(e) => {
            eprintln!("读取脚本失败: {}", e);
            (StatusCode::NOT_FOUND, format!("读取脚本失败: {}", e)).into_response()
        }
    }
}

async fn report_result(ConnectInfo(addr): ConnectInfo<SocketAddr>, Json(payload): Json<Value>) -> impl IntoResponse {
    let ip = addr.ip().to_string();
    let output_dir = ensure_output_dir("output/windows").expect("创建目录失败");
    let filename = format!("{}.json", ip);
    let filepath = output_dir.join(filename);
    println!("已获取客户端: {}数据", ip);
    match serde_json::to_string_pretty(&payload) {
        Ok(pretty) => {
            if let Err(e) = fs::write(&filepath, pretty).await {
                eprintln!("写入文件失败: {}", e);
            }
        }
        Err(e) => {
            eprintln!("JSON格式化错误: {}", e);
        }
    }

    "报告接收成功"
}

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

#[derive(Debug, Args)]
pub struct WindowsArgs {
    #[arg(short, long)]
    pub file: Option<String>,
}

pub async fn run(args: &WindowsArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let script_path = args.file.clone().unwrap_or_else(|| "windows.ps1".to_string());

    let app = Router::new()
        .route("/script", get(get_script))
        .with_state(Arc::new(script_path))
        .route("/report", post(report_result));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
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
    println!("客户端IP: {}", ip);
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

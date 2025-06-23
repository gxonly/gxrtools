use crate::constants::load_commands_from_yaml;
use crate::utils::{create_excel_template, ensure_output_dir};
use calamine::{open_workbook, Reader, Xlsx};
use clap::Parser;
use serde_json::{json, Value};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use tokio::task;
use oracle::{Connection, Error as OracleError};

use std::env;
use std::path::PathBuf;
/// 尝试设置 Oracle 客户端路径（失败也不会 panic）
pub fn try_set_oracle_client_path() -> Result<(), String> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let instantclient_dir = exe_dir.join("instantclient");
    if !instantclient_dir.exists() {
        return Err(format!("未找到 Oracle instantclient 路径: {}", instantclient_dir.display()));
    }

    let old_path = std::env::var("PATH").unwrap_or_default();

    unsafe {
        env::set_var("PATH", format!("{};{}", instantclient_dir.display(), old_path));
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(about = "Oracle 安全配置采集工具", long_about = None)]
pub struct OracleArgs {
    #[arg(short = 'H', long, conflicts_with = "file")]
    pub host: Option<String>,

    #[arg(short = 'f', long, conflicts_with = "host")]
    pub file: Option<String>,

    #[arg(short = 'P', long, default_value = "1521", requires = "host")]
    pub port: u16,

    #[arg(short = 'u', long, default_value = "system", requires = "host")]
    pub username: String,

    #[arg(short = 'p', long, requires = "host")]
    pub password: String,

    #[arg(short = 's', long, default_value = "ORCL", requires = "host")]
    pub service_name: String,

    #[arg(long, default_value = "cmd.yaml")]
    pub yaml: String,

    #[arg(short = 'c', long, num_args = 1..)]
    pub commands: Vec<String>,

    #[arg(short = 't', long, default_value = "4")]
    pub threads: usize,

    #[arg(short = 'e', long, requires = "commands")]
    pub echo: bool,
}

#[derive(Debug, Clone)]
pub struct DbInstanceInfo {
    host: String,
    port: u16,
    username: String,
    password: String,
    service_name: String,
}

pub async fn run(args: &OracleArgs) -> Result<(), Box<dyn Error + Send + Sync>> {
    let start_time = Instant::now();

    let db_list = if let Some(file) = &args.file {
        read_hosts_from_excel(file)?
    } else {
        vec![DbInstanceInfo {
            host: args.host.clone().unwrap(),
            port: args.port,
            username: args.username.clone(),
            password: args.password.clone(),
            service_name: args.service_name.clone(),
        }]
    };

    ensure_output_dir("output/oracle")?;
    println!("开始执行，共 {} 个实例", db_list.len());

    let queries = if args.commands.is_empty() {
        load_commands_from_yaml(&args.yaml, "oracle_commands")
    } else {
        args.commands.clone()
    };

    let mut handles = vec![];

    for db in db_list {
        let cmds = queries.clone();
        let echo = args.echo;
        handles.push(task::spawn(async move {
            let result = match connect_and_execute(&db, &cmds, echo).await {
                Ok(json) => json,
                Err(e) => json!({
                    "host": db.host,
                    "error": e.to_string(),
                }),
            };
            let filename = format!("output/oracle/{}.json", db.host.replace(".", "_"));
            let mut file = File::create(filename).unwrap();
            file.write_all(serde_json::to_string_pretty(&result).unwrap().as_bytes())
                .unwrap();
        }));
    }

    for h in handles {
        h.await?;
    }

    println!(
        "✅ 所有任务完成，用时 {:.2}s",
        start_time.elapsed().as_secs_f64()
    );
    Ok(())
}

pub async fn connect_and_execute(
    db: &DbInstanceInfo,
    commands: &[String],
    echo: bool,
) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let mut output = serde_json::Map::new();

    for cmd in commands {
        let db = db.clone();
        let cmd = cmd.clone();
        let host_clone = db.host.clone();
        let cmd_for_log = cmd.clone();
        let result = task::spawn_blocking(move || -> Result<Vec<String>, OracleError> {
            let conn = Connection::connect(
                &db.username,
                &db.password,
                &format!("{}:{}/{}", db.host, db.port, db.service_name),
            )?;

            let mut stmt = conn.statement(&cmd).build()?;
            let rows = stmt.query(&[])?;

            let mut formatted = Vec::new();
            let column_info = rows.column_info();
            let column_count = column_info.len();

            for row_result in rows {
                let row = row_result?;
                let mut line = Vec::new();
                for i in 0..column_count {
                    let val: String = row.get::<usize, Option<String>>(i)?.unwrap_or_else(|| "NULL".to_string());
                    line.push(val);
                }
                formatted.push(line.join(" | "));
            }

            Ok(formatted)
        }).await??;

        if echo {
            println!("[{}] {}", host_clone, cmd_for_log);
            for line in &result {
                println!("{}", line);
            }
        }

        output.insert(
            cmd_for_log.clone(),
            json!({
                "status": "✅ 成功",
                "output": result.join("\n"),
            }),
        );
    }

    Ok(json!({
        "host": db.host,
        "results": output,
    }))
}

fn read_hosts_from_excel<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<DbInstanceInfo>, Box<dyn Error + Send + Sync>> {
    if !path.as_ref().exists() {
        let _ = create_excel_template(
            &path,
            vec![
                "主机地址".to_string(),
                "端口".to_string(),
                "用户名".to_string(),
                "密码".to_string(),
                "服务名".to_string(),
            ],
        );
        println!("模板已生成，请填写后重新运行：{}", path.as_ref().display());
        std::process::exit(1);
    }

    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range("Sheet1")
        .ok_or("找不到 Sheet1")??;

    let mut hosts = vec![];

    for row in range.rows().skip(1) {
        if row.len() < 5 {
            continue;
        }

        let host = row[0].get_string().unwrap_or("").to_string();
        let port = row[1].get_float().unwrap_or(1521.0) as u16;
        let username = row[2].get_string().unwrap_or("system").to_string();
        let password = row[3].get_string().unwrap_or("").to_string();
        let service_name = row[4].get_string().unwrap_or("ORCL").to_string();

        if host.is_empty() || password.is_empty() {
            continue;
        }

        hosts.push(DbInstanceInfo {
            host,
            port,
            username,
            password,
            service_name,
        });
    }

    if hosts.is_empty() {
        return Err("Excel 中没有有效的主机数据".into());
    }

    Ok(hosts)
}

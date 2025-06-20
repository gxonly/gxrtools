use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;


pub fn load_commands_from_yaml(path: &str, command_type: &str) -> Vec<String> {
    if !Path::new(path).exists() {
        eprintln!("❌ YAML 文件不存在: {}", path);
        return vec![];
    }

    let content = fs::read_to_string(path)
        .expect("❌ 无法读取 YAML 文件");

    let config:  HashMap<String, Vec<String>> = match serde_yaml::from_str(&content) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("❌ YAML 解析失败: {}", err);
            return vec![];
        }
    };

    match config.get(command_type) {
        Some(cmds) => cmds.clone(),
        None => {
            eprintln!("⚠️ 未找到命令类型 \"{}\"", command_type);
            let available: Vec<&String> = config.keys().collect();
            eprintln!("✅ 可用的命令类型有: {:?}", available);
            vec![]
        }
    }
}

// src/constants.rs

// pub fn default_ssh_commands() -> Vec<String> {
//     vec![
//         "df -h".to_string(),
//         "uptime".to_string(),
//         "whoami".to_string(),
//     ]
// }


use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct CommandConfig {
    ssh_commands: Vec<String>,
}

pub fn default_ssh_commands() -> Vec<String> {
    let path: &str = "cmd.yaml";
    let content = fs::read_to_string(path)
        .expect("Failed to read YAML file");

    let config: CommandConfig = serde_yaml::from_str(&content)
        .expect("Failed to parse YAML file");

    config.ssh_commands
}

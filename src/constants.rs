use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct CommandConfig {
    ssh_commands: Vec<String>,
    mysql_commands: Vec<String>,
}

pub fn default_ssh_commands() -> Vec<String> {
    let path: &str = "cmd.yaml";
    let content = fs::read_to_string(path)
        .expect("Failed to read YAML file");

    let config: CommandConfig = serde_yaml::from_str(&content)
        .expect("Failed to parse YAML file");

    config.ssh_commands
}

pub fn default_mysql_commands() -> Vec<String> {
    let path: &str = "cmd.yaml";
    let content = fs::read_to_string(path)
        .expect("Failed to read YAML file");

    let config: CommandConfig = serde_yaml::from_str(&content)
        .expect("Failed to parse YAML file");

    config.mysql_commands
}
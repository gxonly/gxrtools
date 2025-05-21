// src/constants.rs

pub fn default_ssh_commands() -> Vec<String> {
    vec![
        "df -h".to_string(),
        "uptime".to_string(),
        "whoami".to_string(),
    ]
}

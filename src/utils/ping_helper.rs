use std::process::Command;

pub fn ping(target: &str) -> Result<String, String> {
    let output = Command::new("ping")
        .arg("-c 4")
        .arg(target)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

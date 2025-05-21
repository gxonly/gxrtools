use std::fs;
use std::path::PathBuf;
use std::error::Error;

//判断文件夹是否存在并创建文件夹
pub fn ensure_output_dir(path: &str) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let output_dir = PathBuf::from(path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }
    Ok(output_dir)
}
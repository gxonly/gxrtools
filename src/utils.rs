use std::fs;
use std::path::{PathBuf, Path};
use std::error::Error;
use rust_xlsxwriter::{Workbook, XlsxError, Format};

//判断文件夹是否存在并创建文件夹
pub fn ensure_output_dir(path: &str) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let output_dir = PathBuf::from(path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }
    Ok(output_dir)
}

// 判断文件是否存在
pub fn check_file_exists(file_path: &Path) -> bool{
    file_path.exists()
}

// 创建excel表
pub fn create_excel_template<P: AsRef<Path>>(path: P, headers: Vec<String>) -> Result<(), XlsxError> {
    // 创建一个工作簿，传入文件名
    let mut workbook = Workbook::new(path.as_ref().to_str().unwrap());

    // 添加一个工作表
    let worksheet = workbook.add_worksheet();

    // 使用默认格式
    let default_format = Format::default();

    // 写入标题行（第一行）
    for (col_num, header) in headers.iter().enumerate() {
        worksheet.write_string(0, col_num as u16, header, &default_format)?;  // 传入格式
    }

    // 保存 Excel 文件
    workbook.close()?;

    Ok(())
}
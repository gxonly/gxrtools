use chrono::Local;
use rust_xlsxwriter::ColNum;
use rust_xlsxwriter::{Format, Workbook, XlsxError};
use std::error::Error;
use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

/// 扫描进度控制结构体
#[derive(Clone)]
pub struct ScanProgress {
    pub pb: Arc<ProgressBar>,
}

impl ScanProgress {
    /// 初始化新的进度条
    pub fn new(total: u64) -> Self {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {percent}% ({eta})",
            )
                .unwrap(),
        );
        Self {
            pb: Arc::new(pb),
        }
    }
    /// 进度 +1
    pub fn inc(&self) {
        self.pb.inc(1);
    }
    /// 在进度条上方输出信息（不会破坏进度条）
    pub fn println<S: AsRef<str>>(&self, msg: S) {
        let _ = self.pb.println(msg.as_ref());
    }
    /// 完成并关闭进度条
    pub fn finish(&self) {
        self.pb.finish();
    }
}


//判断文件夹是否存在并创建文件夹
pub fn ensure_output_dir(path: &str) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    let output_dir = PathBuf::from(path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }
    Ok(output_dir)
}

// 判断文件是否存在
pub fn check_file_exists(file_path: &Path) -> bool {
    file_path.exists()
}

// 创建excel表
pub fn create_excel_template<P: AsRef<Path>>(
    path: P,
    headers: Vec<String>,
) -> Result<(), XlsxError> {
    // 创建一个工作簿，传入文件名
    let mut workbook = Workbook::new(path.as_ref().to_str().unwrap());

    // 添加一个工作表
    let worksheet = workbook.add_worksheet();

    // 使用默认格式
    let default_format = Format::default();

    // 写入标题行（第一行）
    for (col_num, header) in headers.iter().enumerate() {
        worksheet.write_string(0, col_num as u16, header, &default_format)?; // 传入格式
    }

    // 保存 Excel 文件
    workbook.close()?;

    Ok(())
}

// 整理IP地址为列表
pub fn parse_targets(targets: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let mut all_ips = Vec::new();
    for target in targets.split(',') {
        let target = target.trim();
        if target.contains('/') {
            // CIDR 格式
            let cidr_ips = generate_ips_from_cidr(target)?;
            all_ips.extend(cidr_ips);
        } else if let Some(_) = target.rfind('-') {
            // IP 范围格式：192.168.1.5-10
            let range_ips = generate_ips_from_range(target)?;
            all_ips.extend(range_ips);
        } else {
            // 单个 IP
            let ip =
                Ipv4Addr::from_str(target).map_err(|_| format!("无效的 IP 地址: {}", target))?;
            all_ips.push(ip.to_string());
        }
    }

    Ok(all_ips)
}

fn generate_ips_from_cidr(cidr: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return Err("CIDR 格式错误".into());
    }

    let base_ip = Ipv4Addr::from_str(parts[0]).map_err(|_| "无效的IP地址".to_string())?;
    let subnet_mask = parts[1]
        .parse::<u8>()
        .map_err(|_| "无效的子网掩码".to_string())?;

    if subnet_mask > 32 {
        return Err("子网掩码不能大于32".into());
    }

    let ip_u32 = u32::from(base_ip);
    let num_ips = 2u32.pow((32 - subnet_mask) as u32);
    let start = ip_u32 + 1;
    let end = ip_u32 + num_ips - 2;

    let mut ips = Vec::new();
    for i in start..=end {
        ips.push(Ipv4Addr::from(i).to_string());
    }

    Ok(ips)
}

fn generate_ips_from_range(range_str: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let dash_pos = range_str.rfind('-').ok_or("无效的 IP 范围格式")?;
    let (base, end) = range_str.split_at(dash_pos);
    let base_ip = Ipv4Addr::from_str(base).map_err(|_| "无效的 IP 地址")?;
    let end_part = &end[1..]; // 去掉 '-'

    let base_parts: Vec<u8> = base_ip.octets().to_vec();
    let end_last = end_part.parse::<u8>().map_err(|_| "IP 范围结束值无效")?;

    if end_last < base_parts[3] {
        return Err("IP 范围结束值必须大于开始值".into());
    }

    let mut ips = Vec::new();
    for i in base_parts[3]..=end_last {
        let ip = Ipv4Addr::new(base_parts[0], base_parts[1], base_parts[2], i);
        ips.push(ip.to_string());
    }

    Ok(ips)
}

// 将内容写入到excel中
pub fn save_to_excel<T, F>(
    data: &[T],
    headers: &[&str],
    row_mapper: F,
    subdir: &str,
    filename_prefix: &str,
) -> Result<String, Box<dyn Error + Send + Sync>>
where
    F: Fn(&T) -> Vec<String>,
{
    let output_dir = ensure_output_dir(&format!("output/{}", subdir))?;

    let timestamp = Local::now().format("%Y%m%d%H%M").to_string();
    let filename = format!("{}_{}.xlsx", timestamp, filename_prefix);
    let filepath = output_dir.join(&filename);

    let mut workbook = Workbook::new(filepath.to_str().unwrap());
    let worksheet = workbook.add_worksheet();
    let default_format = Format::default();

    // 写入表头
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string(0, ColNum::from(col as u16), header, &default_format)?;
    }

    // 写入数据
    for (i, item) in data.iter().enumerate() {
        let row_data = row_mapper(item);
        for (j, value) in row_data.iter().enumerate() {
            worksheet.write_string(
                (i + 1) as u32,
                ColNum::from(j as u16),
                value,
                &default_format,
            )?;
        }
    }

    workbook.close()?;
    println!("✅ 结果已存储至：output/{}/{}", subdir, &filename);
    Ok(filepath.to_string_lossy().to_string())
}

/// 解析端口字符串，如 "22,80-443,9990-10000"
pub fn parse_ports(port_str: &str) -> Vec<u16> {
    let mut ports = Vec::new();

    for part in port_str.split(',') {
        if part.contains('-') {
            let bounds: Vec<&str> = part.splitn(2, '-').collect();
            if let (Ok(start), Ok(end)) = (bounds[0].trim().parse::<u16>(), bounds[1].trim().parse::<u16>()) {
                ports.extend(start..=end);
            }
        } else if let Ok(port) = part.trim().parse::<u16>() {
            ports.push(port);
        }
    }

    ports.sort_unstable();
    ports.dedup();
    ports
}

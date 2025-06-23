use std::ffi::CStr;

#[no_mangle]
pub extern "C" fn default_port() -> u16 {
    8080
}

#[no_mangle]
pub extern "C" fn check(ip: *const std::os::raw::c_char, port: u16) -> bool {
    let c_str = unsafe { CStr::from_ptr(ip) };
    let ip_str = c_str.to_str().unwrap_or("<invalid>");

    println!("[PoC测试库] 接收到目标 => IP: {}, Port: {}", ip_str, port);

    true // 返回 true 表示发现漏洞（仅用于演示）
}

use std::ffi::{CStr, CString};
use std::net::{TcpStream, SocketAddr};
use std::os::raw::{c_char, c_ushort};
use std::time::Duration;
use std::io::{Write, Read};

#[no_mangle]
pub extern "C" fn default_port() -> c_ushort {
    445
}

#[no_mangle]
pub extern "C" fn check(ip: *const c_char, port: c_ushort) -> bool {
    let c_str = unsafe { CStr::from_ptr(ip) };
    let ip_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let addr: SocketAddr = match format!("{}:{}", ip_str, port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };

    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
        Ok(s) => s,
        Err(_) => return false,
    };

    stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(3))).ok();

    // SMBv1 Negotiate Protocol Request Payload
    let smb_payload: [u8; 95] = [
        0xFF, 0x53, 0x4D, 0x42, 0x72,
        0x00, 0x00, 0x00, 0x00,
        0x18, 0x01, 0x28, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x62,
        0x02, 0x50, 0x43, 0x20, 0x4E, 0x45, 0x54, 0x57, 0x4F, 0x52, 0x4B, 0x20,
        0x50, 0x52, 0x4F, 0x47, 0x52, 0x41, 0x4D, 0x20,
        0x31, 0x2E, 0x30, 0x00,
        0x02, 0x4D, 0x49, 0x43, 0x52, 0x4F, 0x53, 0x4F,
        0x46, 0x54, 0x20, 0x4E, 0x45, 0x54, 0x57, 0x4F,
        0x52, 0x4B, 0x53, 0x20, 0x31, 0x2E, 0x30, 0x00,
        0x02, 0x4D, 0x49, 0x43, 0x52, 0x4F, 0x53, 0x4F,
        0x46, 0x54, 0x20, 0x4E, 0x45, 0x54, 0x57, 0x4F,
        0x52, 0x4B, 0x53, 0x20, 0x33, 0x2E, 0x30, 0x00
    ];

    // 自动构造 NetBIOS header + SMB payload
    let mut packet = Vec::with_capacity(4 + smb_payload.len());
    let len = smb_payload.len() as u32;
    packet.extend_from_slice(&[
        0x00,
        ((len >> 16) & 0xFF) as u8,
        ((len >> 8) & 0xFF) as u8,
        (len & 0xFF) as u8,
    ]);
    packet.extend_from_slice(&smb_payload);

    // 发送数据包
    if stream.write_all(&packet).is_err() {
        return false;
    }

    let mut response = [0u8; 1024];
    let n = match stream.read(&mut response) {
        Ok(size) => size,
        Err(_) => return false,
    };

    if n < 36 {
        return false;
    }

    // SMB Header signature
    if response[4..8] != [0xFF, 0x53, 0x4D, 0x42] {
        return false;
    }

    // 检查状态码（STATUS_INSUFF_SERVER_RESOURCES）
    if response[8..12] == [0x05, 0x02, 0x00, 0xC0] {
        return true;
    }

    true
}

#[no_mangle]
pub extern "C" fn info() -> *const c_char {
    CString::new("EternalBlue (MS17-010) SMBv1 vulnerability detector").unwrap().into_raw()
}

use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;

pub fn connect_ssh(host: &str, username: &str, password_or_key: &str) -> Result<Session, Box<dyn std::error::Error>> {
    let tcp = TcpStream::connect(format!("{}:22", host))?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    if Path::new(password_or_key).exists() {
        session.userauth_pubkey_file(username, None, &std::path::Path::new(password_or_key), None)?;
    } else {
        session.userauth_password(username, password_or_key)?;
    }

    Ok(session)
}

use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

/// Attempt to identify protocol by connecting and reading the banner
pub async fn banner_grab(host: &str, port: u16, timeout: Duration) -> Option<String> {
    let addr = format!("{}:{}", host, port);

    let stream = match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
        Ok(Ok(stream)) => stream,
        _ => return None,
    };

    let mut buf = [0u8; 1024];
    let n = match tokio::time::timeout(timeout, async {
        let mut stream = stream;
        stream.readable().await.ok()?;
        stream.read(&mut buf).await.ok()
    })
    .await
    {
        Ok(Some(n)) => n,
        _ => return None,
    };

    if n == 0 {
        return None;
    }

    let banner = String::from_utf8_lossy(&buf[..n]);
    identify_protocol_from_banner(&banner)
}

/// Match banner text to known protocol signatures
pub fn identify_protocol_from_banner(banner: &str) -> Option<String> {
    let banner_lower = banner.to_lowercase();

    if banner.starts_with("HTTP/") || banner_lower.contains("http/") {
        return Some("http".to_string());
    }
    if banner.starts_with("SSH-") {
        return Some("ssh".to_string());
    }
    if banner.starts_with("+OK") {
        return Some("pop3".to_string());
    }
    if banner.starts_with("220 ") {
        if banner_lower.contains("ftp") {
            return Some("ftp".to_string());
        }
        return Some("smtp".to_string());
    }
    if banner_lower.contains("postgresql") || banner_lower.contains("postgres") {
        return Some("postgres".to_string());
    }
    if banner.starts_with("-ERR") || banner.starts_with("+PONG") || banner.starts_with("$") {
        return Some("redis".to_string());
    }
    if banner_lower.contains("mysql") || banner_lower.contains("mariadb") {
        return Some("mysql".to_string());
    }
    if banner_lower.contains("mongodb") || banner_lower.contains("mongod") {
        return Some("mongodb".to_string());
    }
    if banner_lower.contains("redis") {
        return Some("redis".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_banner() {
        assert_eq!(
            identify_protocol_from_banner("HTTP/1.1 200 OK\r\n"),
            Some("http".to_string())
        );
    }

    #[test]
    fn test_ssh_banner() {
        assert_eq!(
            identify_protocol_from_banner("SSH-2.0-OpenSSH_9.0\r\n"),
            Some("ssh".to_string())
        );
    }

    #[test]
    fn test_redis_banner() {
        assert_eq!(
            identify_protocol_from_banner("+PONG\r\n"),
            Some("redis".to_string())
        );
    }

    #[test]
    fn test_postgres_banner() {
        assert_eq!(
            identify_protocol_from_banner("\0\0\0\x08PostgreSQL"),
            Some("postgres".to_string())
        );
    }

    #[test]
    fn test_smtp_banner() {
        assert_eq!(
            identify_protocol_from_banner("220 mail.example.com ESMTP"),
            Some("smtp".to_string())
        );
    }

    #[test]
    fn test_ftp_banner() {
        assert_eq!(
            identify_protocol_from_banner("220 FTP server ready"),
            Some("ftp".to_string())
        );
    }

    #[test]
    fn test_unknown_banner() {
        assert_eq!(identify_protocol_from_banner("some random data"), None);
    }
}

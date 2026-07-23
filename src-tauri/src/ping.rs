use std::net::TcpStream;
use std::time::{Duration, Instant};

/// TCP ping to host:port with timeout. Returns ms latency.
pub async fn tcp_ping(host: &str, timeout_ms: u64) -> Result<u64, String> {
    let host = host.to_string();
    let port = 443; // Try HTTPS port — ShadowTLS listens on 8553 but we ping common port
    let timeout = Duration::from_millis(timeout_ms);

    let start = Instant::now();
    match TcpStream::connect_timeout(&format!("{}:{}", host, port).parse().unwrap(), timeout) {
        Ok(_) => {
            let ms = start.elapsed().as_millis() as u64;
            Ok(ms)
        }
        Err(_) => {
            // Fallback: try the actual ShadowTLS port
            let start2 = Instant::now();
            match TcpStream::connect_timeout(
                &format!("{}:8553", host).parse().unwrap(),
                timeout,
            ) {
                Ok(_) => Ok(start2.elapsed().as_millis() as u64),
                Err(e) => Err(format!("ping to {} failed: {}", host, e)),
            }
        }
    }
}

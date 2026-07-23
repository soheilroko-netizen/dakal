use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// TCP ping to a server:port. Returns latency in ms, 0 on failure.
pub async fn tcp_ping(addr: &str, timeout_ms: u64) -> u64 {
    let start = Instant::now();
    match timeout(Duration::from_millis(timeout_ms), TcpStream::connect(addr)).await {
        Ok(Ok(_)) => start.elapsed().as_millis() as u64,
        _ => 0,
    }
}

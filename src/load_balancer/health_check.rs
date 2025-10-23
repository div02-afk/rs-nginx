use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};

async fn check_health_single(address: &str, path: &str) -> Result<bool, std::io::Error> {
    // Connect to the backend server
    let mut stream = TcpStream::connect(address).await?;

    // Send a simple HTTP GET request
    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, address
    );

    stream.write_all(request.as_bytes()).await?;
    stream.flush().await?;

    // Read the response (just the first part to check status)
    let mut buffer = [0u8; 512];
    let n = stream.read(&mut buffer).await?;

    if n == 0 {
        return Ok(false); // No response
    }

    // Parse the response to check for "HTTP/1.1 200" or "HTTP/1.0 200"
    let response = String::from_utf8_lossy(&buffer[..n]);
    let is_healthy = response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200");

    Ok(is_healthy)
}

/// Performs a simple HTTP health check on the given address
/// Returns true if the server responds with HTTP 200, false otherwise
async fn health_probe(address: &String, path: &str) -> bool {
    // println!("Checking health for {}", address);
    // Set a timeout for the health check (5 seconds)
    let result = timeout(Duration::from_secs(5), check_health_single(address, path)).await;

    match result {
        Ok(Ok(healthy)) => healthy,
        Ok(Err(_)) => false, // Connection error
        Err(_) => false,     // Timeout
    }
}

pub fn check_health(addresses: Vec<String>, path: String, health_result: Arc<RwLock<Vec<bool>>>) {
    // println!("checking health");
    tokio::spawn(async move {
        let addr_clone = addresses.clone();
        loop {
            for (i, address) in addr_clone.iter().enumerate() {
                let result = health_probe(address, &path).await;
                let mut health_result_lock = health_result.write().await;
                health_result_lock[i] = result;
                drop(health_result_lock);
            }
            sleep(Duration::from_secs(10)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_probe() {
        // Test with a known good endpoint (assuming test servers are running)
        let result = health_probe(&"127.0.0.1:3001".to_string(), "/health").await;
        println!("Health check result: {}", result);
    }
}

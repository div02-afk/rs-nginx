use std::{ io::{ Error, Write }, path::{ Path, PathBuf } };

use crate::config::ServerConfig;
use tokio::io::AsyncWriteExt;
use tokio::{ fs, io::AsyncReadExt, net::{ TcpListener, TcpStream } };

pub async fn listen(config: &ServerConfig) -> Result<(), Error> {
    let addr = format!("0.0.0.0:{}", config.listen);
    let tcp_listener = TcpListener::bind(addr).await?;
    println!("listening on port {}", config.listen);
    if config.root.is_some() {
        let temp_root = config.root.clone().unwrap();
        let root_dir = PathBuf::from(temp_root);
        loop {
            let (mut stream, addr) = tcp_listener.accept().await?;
            let root_dir_clone = root_dir.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_static_files(&mut stream, &root_dir_clone).await {
                    eprintln!("Error handling {}: {}", addr, e);
                }
            });
        }
    }
    Ok(())
}

pub async fn handle_static_files(stream: &mut TcpStream, root: &PathBuf) -> Result<(), Error> {
    let mut buff = [0; 1024];
    let n = match stream.read(&mut buff).await {
        Ok(n) if n == 0 => {
            return Err(Error::other("Empty buffer"));
        }
        Ok(n) => n,
        Err(e) => {
            eprintln!("Failed to read: {}", e);
            return Err(Error::other(format!("Error {}", e)));
        }
    };

    let request = String::from_utf8_lossy(&buff[..n]);
    let request_line = request.lines().next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let requested_path = parts.next().unwrap_or("/");

    println!("Method {}, Path {}", method, requested_path);

    if let Some(path) = safe_path(root, requested_path) {
        let file_result = fs::File::open(&path).await;
        if file_result.is_ok() {
            let mut file = file_result.unwrap();
            println!("file size: {:?}", file.metadata().await.unwrap().len());
            let mut contents = Vec::new();
            let _ = file.read_to_end(&mut contents).await.unwrap();

            let content_type = if path.extension().and_then(|s| s.to_str()) == Some("html") {
                "text/html"
            } else if path.extension().and_then(|s| s.to_str()) == Some("css") {
                "text/css"
            } else if path.extension().and_then(|s| s.to_str()) == Some("js") {
                "application/javascript"
            } else {
                "application/octet-stream"
            };

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
                contents.len(),
                content_type
            );

            // Send headers
            stream.write_all(response.as_bytes()).await.unwrap();

            // Send file contents
            stream.write_all(&contents).await.unwrap();
            stream.flush().await.unwrap();

            println!("Ok");
            return Ok(());
        } else {
            return Err(Error::other("File not found"));
        }
    }

    let response = format!(
        "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\nContent-Type: {}\r\nConnection: close\r\n\r\n",
        "",
        ""
    );

    let _ = stream.write(response.as_bytes()).await;
    let _ = stream.flush().await;
    return Err(Error::other("Invalid path requested"));
}

fn safe_path(root: &PathBuf, requested_path: &str) -> Option<PathBuf> {
    let requested_path = requested_path.trim_start_matches(|c| (c == '/' || c == '\\'));
    let path = root.as_path().join(requested_path);
    // println!("root {:?},requested {}, pathbuf {:?}", root, requested_path, path);
    if let (Ok(path), Ok(canon_root)) = (path.canonicalize(), root.canonicalize()) {
        if path.starts_with(&canon_root) {
            return Some(path);
        } else {
            eprintln!(
                "Reqested Path {:?} doesn't start with root {:?}",
                requested_path,
                canon_root
            );
        }
    }

    return None;
}

use std::{
    io::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    cache::lru::Cache,
    response_builder::http::{BAD_REQUEST_RESPONSE, NOT_FOUND_RESPONSE, create_response},
};

pub async fn handle_static_files(
    stream: &mut TcpStream,
    root: &Path,
    cache: &Arc<Cache>,
) -> Result<(), Error> {
    let mut buff = [0; 1024];

    let n = match stream.read(&mut buff).await {
        Ok(0) => {
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

    //checking cached response

    if let Some(path) = safe_path(root, requested_path) {
        if method.to_lowercase().eq("get")
            && let Some(data) = cache.get(&path).await
        {
            stream
                .write_all(create_response(&data, &path).as_bytes())
                .await
                .unwrap();
            // Send file contents
            stream.write_all(&data).await.unwrap();
            stream.flush().await.unwrap();

            println!("Cached Ok");
            return Ok(());
        }
        let file_result = fs::File::open(&path).await;
        if let Ok(file) = file_result {
            let mut file = file;
            println!("file size: {:?}", file.metadata().await.unwrap().len());
            let mut contents = Vec::new();
            let _ = file.read_to_end(&mut contents).await.unwrap();
            cache.add(&path, &contents).await;
            // Send headers
            stream
                .write_all(create_response(&contents, &path).as_bytes())
                .await
                .unwrap();

            // Send file contents
            stream.write_all(&contents).await.unwrap();
            stream.flush().await.unwrap();

            println!("Ok");
            return Ok(());
        } else {
            // Send 404 response before closing

            let _ = stream.write_all(NOT_FOUND_RESPONSE).await;
            let _ = stream.flush().await;
            let _ = stream.shutdown().await;

            return Err(Error::other("File not found"));
        }
    }

    // Invalid path - send 400 and close

    let _ = stream.write_all(BAD_REQUEST_RESPONSE).await;
    let _ = stream.flush().await;
    let _ = stream.shutdown().await;

    Err(Error::other("Invalid path requested"))
}

fn safe_path(root: &Path, requested_path: &str) -> Option<PathBuf> {
    let requested_path = requested_path.trim_start_matches(['/', '\\']);
    let path = root.join(requested_path);
    // println!("root {:?},requested {}, pathbuf {:?}", root, requested_path, path);
    if let (Ok(path), Ok(canon_root)) = (path.canonicalize(), root.canonicalize()) {
        if path.starts_with(&canon_root) {
            return Some(path);
        } else {
            eprintln!(
                "Reqested Path {:?} doesn't start with root {:?}",
                requested_path, canon_root
            );
        }
    }

    None
}

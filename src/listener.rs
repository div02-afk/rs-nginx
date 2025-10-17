use std::{ io::{ Error }, path::{ PathBuf } };

use crate::{
    config::ServerConfig,
    handler::{ proxy_handler::handle_proxy, static_handler::handle_static_files },
};
use tokio::{ io::AsyncWriteExt, net::TcpListener };

pub async fn listen(config: &ServerConfig) -> Result<(), Error> {
    let addr = format!("0.0.0.0:{}", config.listen);
    let tcp_listener = TcpListener::bind(addr).await?;
    println!("listening on port {}", config.listen);
    if config.root.is_some() {
        let temp_root = config.root.clone().unwrap();
        let root_dir = PathBuf::from(temp_root);
        loop {
            let (mut stream, addr) = tcp_listener.accept().await?;
            println!("Received Static file request");
            let root_dir_clone = root_dir.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_static_files(&mut stream, &root_dir_clone).await {
                    eprintln!("Error handling {}: {}", addr, e);
                    
                }
            });
        }
    } else if config.proxy.is_some() {
        let proxy_addr = config.proxy.clone().unwrap();
        loop {
            let (mut stream, addr) = tcp_listener.accept().await?;
            let proxy_addr_clone = proxy_addr.clone();
            println!("Received Proxy request");
            tokio::spawn(async move {
                if let Err(e) = handle_proxy(&mut stream, &proxy_addr_clone).await {
                    eprintln!("Error handling {}: {}", addr, e);
                    let _ = stream.shutdown().await;
                }
            });
        }
    }
    Ok(())
}

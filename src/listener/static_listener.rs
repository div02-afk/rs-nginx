use std::{io::Error, path::PathBuf, sync::Arc};

use tokio::net::TcpListener;

use crate::{
    cache::lru::Cache, config::ServerConfig, handler::static_handler::handle_static_files,
};

pub async fn static_listener(
    config: &ServerConfig,
    tcp_listener: &TcpListener,
    cache: &Arc<Cache>,
) -> Result<(), Error> {
    let temp_root = config.root.clone().unwrap();
    let root_dir = PathBuf::from(temp_root);
    loop {
        let (mut stream, addr) = tcp_listener.accept().await?;
        print!("Received Static file request : ");
        let root_dir_clone = root_dir.clone();
        let cloned_cache = cache.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_static_files(&mut stream, &root_dir_clone, &cloned_cache).await {
                eprintln!("Error handling {}: {}", addr, e);
            }
        });
    }
}

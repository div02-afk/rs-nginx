use crate::{ cache::lru::Cache, config::read_config, listener::listen };
use std::{ path::Path, sync::Arc };

mod cache;
mod config;
mod handler;
mod listener;
mod response_builder;

#[tokio::main]
async fn main() {
    let cache = Cache::new(1024);

    let config_path = Path::new("config.yaml");
    if !config_path.exists() {
        panic!("Config not found");
    }

    let config = read_config(config_path);
    let config = Arc::new(config);
    for server in &config.http {
        let server = server.clone(); // Clone before moving into async block
        tokio::spawn(async move {
            if let Err(e) = listen(&server).await {
                eprintln!("Error on port {}: {}", server.listen, e);
            }
        });
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}

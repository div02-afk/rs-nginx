use std::{ io::Error, path::PathBuf, sync::Arc, thread::sleep, time::Duration };

use crate::{
    cache::lru::Cache,
    config::{ ProxyType, ServerConfig },
    handler::{ proxy_handler::handle_proxy, static_handler::handle_static_files },
    load_balancer::{ health_check::check_health, round_robin::get_next_server },
};
use tokio::{ io::AsyncWriteExt, net::TcpListener, sync::RwLock };

pub async fn listen(config: &ServerConfig) -> Result<(), Error> {
    let addr = format!("0.0.0.0:{}", config.listen);
    let tcp_listener = TcpListener::bind(addr).await?;
    println!("listening on port {}", config.listen);
    let cache = Arc::new(Cache::new(config.cache.unwrap_or(0)));

    //cache initialized
    if config.root.is_some() {
        let temp_root = config.root.clone().unwrap();
        let root_dir = PathBuf::from(temp_root);
        loop {
            let (mut stream, addr) = tcp_listener.accept().await?;
            print!("Received Static file request : ");
            let root_dir_clone = root_dir.clone();
            let cloned_cache = cache.clone();
            tokio::spawn(async move {
                if
                    let Err(e) = handle_static_files(
                        &mut stream,
                        &root_dir_clone,
                        &cloned_cache
                    ).await
                {
                    eprintln!("Error handling {}: {}", addr, e);
                }
            });
        }
    } else if config.proxy.is_some() {
        let proxy_addr = config.proxy.clone().unwrap();
        match proxy_addr {
            ProxyType::Single(proxy_addr) =>
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

            ProxyType::Multiple(proxy_addr) => {
                let mut current = 0;
                let proxy_size = proxy_addr.len();
                let health_result = Arc::new(RwLock::new(vec![true; proxy_size]));
                if let Some(health_path) = &config.proxy_health {
                    check_health(proxy_addr.clone(), health_path.clone(), health_result.clone());
                }
                loop {
                    let (mut stream, addr) = tcp_listener.accept().await?;
                    let mut iter_count: usize = 0;
                    let mut fail_count: usize = 0;
                    loop {
                        if iter_count > proxy_size {
                            iter_count = 0;
                            fail_count += 1;

                            if fail_count > 3 {
                                let _ = stream.shutdown().await;
                            }
                            sleep(Duration::from_secs(2));
                        }

                        current = get_next_server(proxy_size, current);
                        if health_result.read().await[current] {
                            break;
                        }

                        iter_count += 1;
                    }
                    let balanced_proxy_address = proxy_addr[current].clone();
                    println!("Received Proxy request, proxying to {}", balanced_proxy_address);
                    tokio::spawn(async move {
                        if let Err(e) = handle_proxy(&mut stream, &balanced_proxy_address).await {
                            eprintln!("Error handling {}: {}", addr, e);
                            let _ = stream.shutdown().await;
                        }
                    });
                }
            }
        }
    }
    Ok(())
}

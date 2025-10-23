use std::{io::Error, path::PathBuf, sync::Arc, thread::sleep, time::Duration};

use crate::{
    cache::lru::Cache,
    config::{ProxyType, ServerConfig},
    constants::strategies::{RANDOM, ROUND_ROBIN},
    handler::{proxy_handler::handle_proxy, static_handler::handle_static_files},
    listener::static_listener::{self, static_listener},
    load_balancer::{health_check::check_health, random, round_robin},
};
use tokio::{io::AsyncWriteExt, net::TcpListener, sync::RwLock};

pub async fn listen(config: &ServerConfig) -> Result<(), Error> {
    let addr = format!("0.0.0.0:{}", config.listen);
    let tcp_listener = TcpListener::bind(addr).await?;
    println!("listening on port {}", config.listen);
    let cache = Arc::new(Cache::new(config.cache.unwrap_or(0)));

    //cache initialized
    if config.root.is_some() {
        let _ = static_listener(config, &tcp_listener, &cache).await;
    } else if config.proxy.is_some() {
        let proxy_addr = config.proxy.clone().unwrap();
        match proxy_addr {
            ProxyType::Single(proxy_addr) => loop {
                let (mut stream, addr) = tcp_listener.accept().await?;
                let proxy_addr_clone = proxy_addr.clone();
                println!("Received Proxy request");
                tokio::spawn(async move {
                    if let Err(e) = handle_proxy(&mut stream, &proxy_addr_clone).await {
                        eprintln!("Error handling {}: {}", addr, e);
                        let _ = stream.shutdown().await;
                    }
                });
            },

            ProxyType::Multiple(proxy_addr) => {
                let mut current = 0;
                let proxy_size = proxy_addr.len();
                println!("Proxy size {} ", proxy_size);
                let health_result = Arc::new(RwLock::new(vec![true; proxy_size]));
                if let Some(health_path) = &config.proxy_health {
                    check_health(
                        proxy_addr.clone(),
                        health_path.clone(),
                        health_result.clone(),
                    );
                }

                let get_next_server: fn(usize, usize) -> usize = get_load_balancer_strategy(config);

                loop {
                    let (mut stream, addr) = tcp_listener.accept().await?;
                    let iter_count: usize = 0;
                    let fail_count: usize = 0;
                    get_healthy_server(
                        &mut current,
                        proxy_size,
                        &health_result,
                        get_next_server,
                        &mut stream,
                        iter_count,
                        fail_count,
                    )
                    .await;
                    let balanced_proxy_address = proxy_addr[current].clone();
                    println!(
                        "Received Proxy request, proxying to {}",
                        balanced_proxy_address
                    );
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

fn get_load_balancer_strategy(config: &ServerConfig) -> fn(usize, usize) -> usize {
    match &config.strategy {
        Some(strategy) => {
            println!("Got strategy {}", strategy);
            match strategy.as_str() {
                ROUND_ROBIN => {
                    println!("RR selected");
                    round_robin::get_next_server
                }
                RANDOM => random::get_next_server,
                _ => {
                    println!("Unknown-stragegy");
                    random::get_next_server
                }
            }
        }
        None => {
            println!("No strategy");
            random::get_next_server
        }
    }
}

async fn get_healthy_server(
    current: &mut usize,
    proxy_size: usize,
    health_result: &Arc<RwLock<Vec<bool>>>,
    get_next_server: fn(usize, usize) -> usize,
    stream: &mut tokio::net::TcpStream,
    mut iter_count: usize,
    mut fail_count: usize,
) {
    loop {
        if iter_count > proxy_size {
            iter_count = 0;
            fail_count += 1;

            if fail_count > 3 {
                let _ = stream.shutdown().await;
            }
            sleep(Duration::from_secs(2));
        }

        *current = get_next_server(proxy_size, *current);
        if health_result.read().await[*current] {
            return;
        }

        iter_count += 1;
    }
}

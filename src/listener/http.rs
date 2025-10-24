use std::{io::Error, ops::Add, sync::Arc, time::Duration};

use crate::{
    cache::lru::Cache,
    config::{ProxyType, ServerConfig},
    constants::strategies::{RANDOM, ROUND_ROBIN, WEIGHTED_ROUND_ROBIN},
    handler::proxy_handler::handle_proxy,
    listener::static_listener::static_listener,
    load_balancer::{
        health_check::check_health,
        strategy::{Context, Random, RoundRobin, Strategy, WeightedRoundRobin},
    },
};
use tokio::{io::AsyncWriteExt, net::TcpListener, sync::RwLock, time::sleep};

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
                let proxy_size = proxy_addr.len();
                // println!("Proxy size {} ", proxy_size);
                let health_result = Arc::new(RwLock::new(vec![true; proxy_size]));
                if let Some(health_path) = &config.proxy_health {
                    check_health(
                        proxy_addr.clone(),
                        health_path.clone(),
                        health_result.clone(),
                    );
                }

                let mut strategy = get_load_balancer_strategy(config);
                let server_context = Context {
                    size: proxy_size,
                    weights: get_server_weights(config, proxy_size),
                };

                loop {
                    let (mut stream, addr) = tcp_listener.accept().await?;
                    let current = get_healthy_server(
                        proxy_size,
                        &health_result,
                        &mut strategy,
                        &server_context,
                    )
                    .await;
                    if current.is_none() {
                        println!("No live server found");
                        stream.shutdown().await.unwrap();
                        continue;
                    }

                    let balanced_proxy_address = proxy_addr[current.unwrap()].clone();
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

fn get_load_balancer_strategy(config: &ServerConfig) -> Box<dyn Strategy + Send + Sync> {
    match &config.strategy {
        Some(strategy) => {
            // println!("Got strategy {}", strategy);
            match strategy.as_str() {
                ROUND_ROBIN => Box::new(RoundRobin { current: 0 }),
                RANDOM => Box::new(Random {}),
                WEIGHTED_ROUND_ROBIN => Box::new(WeightedRoundRobin {
                    current: 0,
                    current_count: 0,
                }),
                _ => {
                    println!("Unknown strategy, proceeding with random");
                    Box::new(Random {})
                }
            }
        }
        None => {
            println!("No strategy, proceeding with random");
            Box::new(Random {})
        }
    }
}

async fn get_healthy_server(
    proxy_size: usize,
    health_result: &Arc<RwLock<Vec<bool>>>,
    strategy: &mut Box<dyn Strategy + Send + Sync>,
    context: &Context,
) -> Option<usize> {
    let mut iter_count = 0;
    let mut fail_count: usize = 0;
    let mut sleep_dur = Duration::from_secs(2);
    loop {
        if iter_count > proxy_size {
            iter_count = 0;
            fail_count += 1;

            if fail_count > 3 {
                return None;
            }
            sleep(sleep_dur).await;
            sleep_dur = sleep_dur.add(Duration::from_secs(1));
        }

        let current = strategy.get_next_server(context);
        if health_result.read().await[current] {
            return Some(current);
        }

        iter_count += 1;
    }
}

fn get_server_weights(config: &ServerConfig, proxy_size: usize) -> Vec<u8> {
    if let Some(weights) = &config.weights {
        let mut result = weights.clone();
        if result.len() < proxy_size {
            // Pad with default weight 1 if not enough weights are provided
            result.extend(vec![1; proxy_size - result.len()]);
        } else if result.len() > proxy_size {
            // Truncate if too many weights are provided
            result.truncate(proxy_size);
        }
        result
    } else {
        vec![1; proxy_size]
    }
}

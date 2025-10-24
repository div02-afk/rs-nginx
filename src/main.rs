use crate::config::execute_config;
use notify::{RecursiveMode, Watcher, recommended_watcher};
use std::{path::Path, sync::mpsc::channel, time::Duration};
mod cache;
mod compression;
mod config;
mod constants;
mod handler;
mod listener;
mod load_balancer;
mod response_builder;

#[tokio::main]
async fn main() {
    let config_path = Path::new("config.yaml");
    if !config_path.exists() {
        panic!("Config not found");
    }
    let mut handles = execute_config(config_path);
    let (tx, rx) = channel();
    let mut config_file_watcher = recommended_watcher(tx).unwrap();
    config_file_watcher
        .watch(config_path, RecursiveMode::Recursive)
        .unwrap();

    let debounce_duration = Duration::from_millis(500);
    let mut last_event_time;

    while let Ok(_event) = rx.recv() {
        last_event_time = std::time::Instant::now();

        // Wait debounce period
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if last_event_time.elapsed() >= debounce_duration {
                println!("Trigger server restart!");
                for handle in handles {
                    handle.abort();
                    let _ = handle.await;
                }
                handles = execute_config(config_path);
                if handles.is_empty() {
                    println!("Invalid Config");
                    return;
                }

                break;
            }
        }
    }

    // loop {
    //     tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    // }
}

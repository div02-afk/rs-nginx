use std::{fs::File, io::Error, path::Path, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::listener::listener::listen;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct StaticFileConfig {
    pub path: String,
    pub dir: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum ProxyType {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ServerConfig {
    pub listen: u16,
    pub cache: Option<usize>,
    pub root: Option<String>,
    pub proxy: Option<ProxyType>,
    pub proxy_health: Option<String>,
    pub strategy: Option<String>,
    pub weights: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub http: Vec<ServerConfig>,
}

pub fn read_config(config_path: &Path) -> Option<Config> {
    let file = File::open(config_path).expect("Config file not found");
    let config = serde_yaml::from_reader(file).expect("Invalid Config format");
    let validation = validate(&config);
    if validation.is_err() {
        println!("{:?}", validation.err());
        return None;
    }
    return Some(config);
}

fn validate(config: &Config) -> Result<(), Error> {
    for server_config in &config.http {
        if let Some(weights) = &server_config.weights
            && let Some(proxy) = &server_config.proxy
        {
            match proxy {
                ProxyType::Multiple(p) => {
                    if weights.len() != p.len() {
                        return Err(Error::other(
                            "Invalid weights: Weight length is not equal to proxy lenght",
                        ));
                    }
                    for weight in weights {
                        if *weight == 0 {
                            return Err(Error::other("Invalid weights: Weights can't be 0"));
                        }
                    }
                }
                ProxyType::Single(_) => {
                    return Err(Error::other(
                        "Invalid Config: weights property is only for multiply proxy addresses",
                    ));
                }
            }
        }
    }

    Ok(())
}

pub fn execute_config(config_path: &Path) -> Vec<JoinHandle<()>> {
    let config = read_config(config_path);
    if config.is_none() {
        return Vec::new();
    }

    let config = Arc::new(config.unwrap());
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for server in &config.http {
        let server = server.clone(); // Clone before moving into async block
        let handle = tokio::spawn(async move {
            if let Err(e) = listen(&server).await {
                eprintln!("Error on port {}: {}", server.listen, e);
            }
        });

        handles.push(handle);
    }

    handles
}

use std::{ fs::File, io::Error, path::Path};

use serde::{ Deserialize, Serialize};

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

pub fn read_config(config_path: &Path) -> Config {
    let file = File::open(config_path).expect("Config file not found");
    let config  = serde_yaml::from_reader(file).expect("Invalid Config format");



    config
}


fn validate(config : &Config) -> Result<(),Error> {
    let config = config.clone();
    for server_config in &config.http {
        if let Some(weights) = &server_config.weights {
            for weight in weights {
                if *weight == 0 {
                   return Err(Error::other("Invalid weights: Weights can't be 0"));
                }
            }
        }
    }

    Ok(())
}

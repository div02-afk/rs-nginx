use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct StaticFileConfig {
    pub path: String,
    pub dir: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq,Clone)]
pub struct ServerConfig {
    pub listen: u16,
    pub root: Option<String>,
    pub proxy: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub http: Vec<ServerConfig>,
}

pub fn read_config(config_path: &Path) -> Config {
    let file = File::open(config_path).expect("Config file not found");
    serde_yaml::from_reader(file).expect("Invalid Config format")
}

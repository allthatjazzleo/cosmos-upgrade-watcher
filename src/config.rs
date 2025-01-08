use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub prometheus: PrometheusConfig,
    pub chain: ChainConfig,
}

#[derive(Deserialize)]
pub struct PrometheusConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct ChainConfig {
    pub watch_list: Vec<String>,
    pub refresh: String,
    pub endpoint: String,
}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let config_str = fs::read_to_string(path).expect("Failed to read config file");
        toml::from_str(&config_str).expect("Failed to parse config file")
    }
}
// Copyright (c) 2025, TheByteSlayer, Triangular
// Stores structured Data in JSON Files and makes it accessible over TCP, written in Rust.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub ip: String,
    pub port: u16,
    pub silent: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            ip: "0.0.0.0".to_string(),
            port: 8080,
            silent: false,
        }
    }
}

impl Config {
    pub fn load_or_create() -> Result<Config, Box<dyn std::error::Error>> {
        let config_path = "triangular-db.toml";
        
        let config = if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path)?;
            match toml::from_str::<Config>(&content) {
                Ok(mut config) => {
                    if config.ip.is_empty() {
                        config.ip = "0.0.0.0".to_string();
                    }
                    if config.port == 0 {
                        config.port = 8080;
                    }
                    config
                }
                Err(_) => {
                    Config::default()
                }
            }
        } else {
            Config::default()
        };
        
        let toml_string = toml::to_string_pretty(&config)?;
        fs::write(config_path, toml_string)?;
        
        Ok(config)
    }
    
    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
} 
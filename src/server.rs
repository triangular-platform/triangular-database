// Copyright (c) 2025, TheByteSlayer, Triangular
// Stores structured Data in JSON Files and makes it accessible over TCP, written in Rust.

mod configuration;
mod api;
mod tree;

use configuration::Config;
use api::start_tcp_server;
use tree::{initialize_tree, create_containers};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load_or_create()?;
    
    initialize_tree()?;
    
    println!("Triangular Database listening on {}", config.address());
    
    create_containers()?;
    
    start_tcp_server(&config.address()).await?;
    
    Ok(())
} 
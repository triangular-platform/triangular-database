// Copyright (c) 2025, TheByteSlayer, Triangular
// Stores structured Data in JSON Files and makes it accessible over TCP, written in Rust.

mod tree;
mod configuration;
mod api;

use tree::initialize_tree;
use configuration::Config;

fn main() {
    let config = Config::load_or_create().unwrap();
    
    if let Err(e) = initialize_tree() {
        if !config.silent {
            eprintln!("Failed to initialize tree: {}", e);
        }
        return;
    }
    
    if let Err(e) = tree::initialize_containers(config.silent) {
        if !config.silent {
            eprintln!("Failed to initialize containers: {}", e);
        }
        return;
    }
    
    if let Err(e) = api::start_server(&config) {
        if !config.silent {
            eprintln!("Server error: {}", e);
        }
    }
} 
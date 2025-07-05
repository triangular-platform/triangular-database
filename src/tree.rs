// Copyright (c) 2025, TheByteSlayer, Triangular
// Stores structured Data in JSON Files and makes it accessible over TCP, written in Rust.

use std::fs;
use std::path::Path;

pub fn initialize_tree() -> Result<(), Box<dyn std::error::Error>> {
    let tree_dir = "tree";
    let tree_file = "tree.json";
    
    if !Path::new(tree_dir).exists() {
        fs::create_dir(tree_dir)?;
    }
    
    if !Path::new(tree_file).exists() {
        fs::write(tree_file, "{}")?;
    }
    
    Ok(())
} 
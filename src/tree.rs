// Copyright (c) 2025, TheByteSlayer, Triangular
// Stores structured Data in JSON Files and makes it accessible over TCP, written in Rust.

use std::fs;
use std::path::Path;
use serde_json::Value;

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

pub fn create_containers() -> Result<(), Box<dyn std::error::Error>> {
    let tree_dir = "tree";
    let tree_file = "tree.json";
    
    let tree_content = fs::read_to_string(tree_file)?;
    let tree_data: Value = serde_json::from_str(&tree_content)?;
    
    if let Value::Object(root_map) = tree_data {
        for (container_name, _) in root_map {
            let container_file_path = format!("{}/{}.json", tree_dir, container_name);
            
            if !Path::new(&container_file_path).exists() {
                let empty_container = serde_json::to_string_pretty(&serde_json::json!({}))?;
                fs::write(&container_file_path, empty_container)?;
                println!("Created Container: {}", container_name);
            }
        }
    }
    
    Ok(())
}

 
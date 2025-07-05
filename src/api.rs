// Copyright (c) 2025, TheByteSlayer, Triangular
// Stores structured Data in JSON Files and makes it accessible over TCP, written in Rust.

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;
use std::fs;
use serde_json::{Value, Map};
use std::path::Path;

pub async fn start_tcp_server(address: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(address).await?;
    
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    
    loop {
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        
        let request = String::from_utf8_lossy(&buffer[..n]);
        let response = process_request(&request).await;
        
        socket.write_all(response.as_bytes()).await?;
    }
    
    Ok(())
}

async fn process_request(request: &str) -> String {
    let request = request.trim();
    let parts: Vec<&str> = request.split_whitespace().collect();
    
    if parts.is_empty() {
        return "Error: Empty command".to_string();
    }
    
    let command = parts[0].to_uppercase();
    
    match command.as_str() {
        "INIT" => {
            if parts.len() >= 3 {
                let container = parts[1];
                let value = parts[2];
                match handle_init_command(container, value).await {
                    Ok(msg) => msg,
                    Err(e) => format!("Error: {}", e),
                }
            } else {
                "Error: INIT requires container and value".to_string()
            }
        }
        "SET" => {
            if parts.len() >= 5 {
                let container = parts[1];
                let module = parts[2];
                let key = parts[3];
                let value = parts[4];
                match handle_set_command(container, module, key, value).await {
                    Ok(msg) => msg,
                    Err(e) => format!("Error: {}", e),
                }
            } else {
                "Error: SET requires container, module, key, and value".to_string()
            }
        }
        "GET" => {
            if parts.len() >= 4 {
                let container = parts[1];
                let module = parts[2];
                let key = parts[3];
                match handle_get_command(container, module, key).await {
                    Ok(msg) => msg,
                    Err(e) => format!("Error: {}", e),
                }
            } else {
                "Error: GET requires container, module, and key".to_string()
            }
        }
        "CONTAINERS" => {
            match handle_containers_command().await {
                Ok(msg) => msg,
                Err(e) => format!("Error: {}", e),
            }
        }
        "MODULES" => {
            if parts.len() >= 2 {
                let container = parts[1];
                match handle_modules_command(container).await {
                    Ok(msg) => msg,
                    Err(e) => format!("Error: {}", e),
                }
            } else {
                "Error: MODULES requires container".to_string()
            }
        }
        "KEYS" => {
            if parts.len() >= 3 {
                let container = parts[1];
                let module = parts[2];
                match handle_keys_command(container, module).await {
                    Ok(msg) => msg,
                    Err(e) => format!("Error: {}", e),
                }
            } else {
                "Error: KEYS requires container and module".to_string()
            }
        }
        _ => format!("Unknown command: {}", command),
    }
}

async fn handle_init_command(container: &str, value: &str) -> Result<String, Box<dyn Error>> {
    // Read tree.json to get the template
    let tree_content = fs::read_to_string("tree.json")?;
    let tree_data: Value = serde_json::from_str(&tree_content)?;
    
    // Find the container in the tree
    if let Value::Object(root_map) = &tree_data {
        if let Some(container_template) = root_map.get(container) {
            // Process the template and replace "module" with the provided value
            let processed_data = process_template(container_template, value)?;
            
            // Write to container.json file
            let container_file_path = format!("tree/{}.json", container);
            let json_output = serde_json::to_string_pretty(&processed_data)?;
            fs::write(&container_file_path, json_output)?;
            
            Ok(format!("INIT {} in Container '{}'", value, container))
        } else {
            Err(format!("Container '{}' not found in tree.json", container).into())
        }
    } else {
        Err("Invalid tree.json format".into())
    }
}

async fn handle_set_command(container: &str, module: &str, key: &str, value: &str) -> Result<String, Box<dyn Error>> {
    // First check if the key exists in tree.json
    let tree_content = fs::read_to_string("tree.json")?;
    let tree_data: Value = serde_json::from_str(&tree_content)?;
    
    // Verify the key exists in tree.json structure
    if let Value::Object(root_map) = &tree_data {
        if let Some(Value::Object(container_obj)) = root_map.get(container) {
            if let Some(Value::Object(module_obj)) = container_obj.get("module") {
                if !module_obj.contains_key(key) {
                    return Err(format!("Key '{}' not defined in tree.json for container '{}'", key, container).into());
                }
            } else {
                return Err(format!("Module structure not found in tree.json for container '{}'", container).into());
            }
        } else {
            return Err(format!("Container '{}' not found in tree.json", container).into());
        }
    }
    
    // Read the container file
    let container_file_path = format!("tree/{}.json", container);
    if !Path::new(&container_file_path).exists() {
        return Err(format!("Container file '{}' does not exist", container_file_path).into());
    }
    
    let container_content = fs::read_to_string(&container_file_path)?;
    let mut container_data: Value = serde_json::from_str(&container_content)?;
    
    // Update the value
    if let Value::Object(root_map) = &mut container_data {
        if let Some(Value::Object(module_obj)) = root_map.get_mut(module) {
            module_obj.insert(key.to_string(), Value::String(value.to_string()));
        } else {
            return Err(format!("Module '{}' not found in container '{}'", module, container).into());
        }
    }
    
    // Write back to file
    let json_output = serde_json::to_string_pretty(&container_data)?;
    fs::write(&container_file_path, json_output)?;
    
    Ok(format!("SET {} {}", key, value))
}

async fn handle_get_command(container: &str, module: &str, key: &str) -> Result<String, Box<dyn Error>> {
    let container_file_path = format!("tree/{}.json", container);
    if !Path::new(&container_file_path).exists() {
        return Err(format!("Container file '{}' does not exist", container_file_path).into());
    }
    
    let container_content = fs::read_to_string(&container_file_path)?;
    let container_data: Value = serde_json::from_str(&container_content)?;
    
    if let Value::Object(root_map) = &container_data {
        if let Some(Value::Object(module_obj)) = root_map.get(module) {
            if let Some(value) = module_obj.get(key) {
                Ok(value.as_str().unwrap_or("").to_string())
            } else {
                Err(format!("Key '{}' not found in module '{}' of container '{}'", key, module, container).into())
            }
        } else {
            Err(format!("Module '{}' not found in container '{}'", module, container).into())
        }
    } else {
        Err("Invalid container file format".into())
    }
}

async fn handle_containers_command() -> Result<String, Box<dyn Error>> {
    let tree_content = fs::read_to_string("tree.json")?;
    let tree_data: Value = serde_json::from_str(&tree_content)?;
    
    if let Value::Object(root_map) = &tree_data {
        let containers: Vec<String> = root_map.keys().cloned().collect();
        Ok(containers.join(", "))
    } else {
        Err("Invalid tree.json format".into())
    }
}

async fn handle_modules_command(container: &str) -> Result<String, Box<dyn Error>> {
    let container_file_path = format!("tree/{}.json", container);
    if !Path::new(&container_file_path).exists() {
        return Err(format!("Container file '{}' does not exist", container_file_path).into());
    }
    
    let container_content = fs::read_to_string(&container_file_path)?;
    let container_data: Value = serde_json::from_str(&container_content)?;
    
    if let Value::Object(root_map) = &container_data {
        let modules: Vec<String> = root_map.keys().cloned().collect();
        Ok(modules.join(", "))
    } else {
        Err("Invalid container file format".into())
    }
}

async fn handle_keys_command(container: &str, module: &str) -> Result<String, Box<dyn Error>> {
    let container_file_path = format!("tree/{}.json", container);
    if !Path::new(&container_file_path).exists() {
        return Err(format!("Container file '{}' does not exist", container_file_path).into());
    }
    
    let container_content = fs::read_to_string(&container_file_path)?;
    let container_data: Value = serde_json::from_str(&container_content)?;
    
    if let Value::Object(root_map) = &container_data {
        if let Some(Value::Object(module_obj)) = root_map.get(module) {
            let keys: Vec<String> = module_obj.keys().cloned().collect();
            Ok(keys.join(", "))
        } else {
            Err(format!("Module '{}' not found in container '{}'", module, container).into())
        }
    } else {
        Err("Invalid container file format".into())
    }
}

fn process_template(template: &Value, replacement_value: &str) -> Result<Value, Box<dyn Error>> {
    match template {
        Value::Object(obj) => {
            let mut new_obj = Map::new();
            for (key, val) in obj {
                if key == "module" {
                    // Replace the "module" key with the replacement value
                    new_obj.insert(replacement_value.to_string(), process_template(val, replacement_value)?);
                } else {
                    new_obj.insert(key.clone(), process_template(val, replacement_value)?);
                }
            }
            Ok(Value::Object(new_obj))
        }
        Value::Array(arr) => {
            let new_arr: Result<Vec<Value>, Box<dyn Error>> = arr.iter()
                .map(|v| process_template(v, replacement_value))
                .collect();
            Ok(Value::Array(new_arr?))
        }
        _ => Ok(template.clone()),
    }
} 
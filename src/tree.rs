// Copyright (c) 2025, TheByteSlayer, Triangular
// Stores structured Data in JSON Files and makes it accessible over TCP, written in Rust.

use std::fs;
use std::path::Path;
use std::thread;
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

static CONTAINER_MANAGER: OnceLock<ContainerManager> = OnceLock::new();

pub struct ContainerManager {
    container_locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
    thread_pool_size: usize,
}

impl ContainerManager {
    pub fn new() -> Self {
        let thread_pool_size = num_cpus::get();
        Self {
            container_locks: Arc::new(Mutex::new(HashMap::new())),
            thread_pool_size,
        }
    }

    pub fn create_containers(&self, silent: bool) -> Result<(), Box<dyn std::error::Error>> {
        let tree_dir = "tree";
        let tree_file = "tree.json";
        
        let tree_content = fs::read_to_string(tree_file)?;
        let tree_data: serde_json::Value = serde_json::from_str(&tree_content)?;
        
        if let serde_json::Value::Object(root_map) = tree_data {
            let containers: Vec<String> = root_map.keys().cloned().collect();
            
            // Use proper multithreading for container creation
            let chunk_size = (containers.len() + self.thread_pool_size - 1) / self.thread_pool_size;
            let chunks: Vec<Vec<String>> = containers.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect();
            
            thread::scope(|s| {
                let handles: Vec<_> = chunks.into_iter().map(|chunk| {
                    s.spawn(move || {
                        for container_name in chunk {
                            let container_file_path = format!("{}/{}.json", tree_dir, container_name);
                            
                            if !Path::new(&container_file_path).exists() {
                                let empty_container = serde_json::to_string_pretty(&serde_json::json!([])).unwrap();
                                if let Err(_) = fs::write(&container_file_path, empty_container) {
                                    if !silent {
                                        eprintln!("Failed to create container: {}", container_name);
                                    }
                                }
                            }
                        }
                    })
                }).collect();
                
                // Wait for all threads to complete
                for handle in handles {
                    let _ = handle.join();
                }
            });
        }
        
        Ok(())
    }

    pub fn get_container_lock(&self, container_name: &str) -> Arc<Mutex<()>> {
        let mut locks = self.container_locks.lock().unwrap();
        locks.entry(container_name.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

pub fn get_container_manager() -> &'static ContainerManager {
    CONTAINER_MANAGER.get_or_init(|| ContainerManager::new())
}

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

pub fn initialize_containers(silent: bool) -> Result<(), Box<dyn std::error::Error>> {
    let manager = get_container_manager();
    manager.create_containers(silent)?;
    Ok(())
}

pub fn handle_init(container: &str, value: &str) -> String {
    // Each JSON operation runs in its own thread context with proper locking
    let manager = get_container_manager();
    let _lock = manager.get_container_lock(container);
    
    // Spawn the actual file operations in a separate thread for better parallelism
    let container_name = container.to_string();
    let value_str = value.to_string();
    
    thread::scope(|s| {
        s.spawn(|| {
            let tree_content = match fs::read_to_string("tree.json") {
                Ok(content) => content,
                Err(_) => return "ERROR: Failed to read tree.json".to_string(),
            };
            
            let tree_data: serde_json::Value = match serde_json::from_str(&tree_content) {
                Ok(data) => data,
                Err(_) => return "ERROR: Failed to parse tree.json".to_string(),
            };
            
            if let Some(template) = tree_data.get(&container_name) {
                let mut new_container = template.clone();
                
                replace_placeholder(&mut new_container, &value_str);
                
                let container_file = format!("tree/{}.json", container_name);
                
                let mut current_data = if Path::new(&container_file).exists() {
                    match fs::read_to_string(&container_file) {
                        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                            Ok(data) => data,
                            Err(_) => serde_json::json!([]),
                        },
                        Err(_) => serde_json::json!([]),
                    }
                } else {
                    serde_json::json!([])
                };
                
                if let Some(array) = current_data.as_array_mut() {
                    array.push(new_container);
                }
                
                let formatted_data = match serde_json::to_string_pretty(&current_data) {
                    Ok(data) => data,
                    Err(_) => return "ERROR: Failed to format data".to_string(),
                };
                
                if let Err(_) = fs::write(&container_file, formatted_data) {
                    return "ERROR: Failed to write container file".to_string();
                }
                
                format!("INIT {} in Container '{}'", value_str, container_name)
            } else {
                "ERROR: Container not found in tree.json".to_string()
            }
        }).join().unwrap_or_else(|_| "ERROR: Thread panic".to_string())
    })
}

pub fn handle_set(container: &str, module: &str, key: &str, value: &str) -> String {
    let manager = get_container_manager();
    let _lock = manager.get_container_lock(container);
    
    let container_name = container.to_string();
    let module_name = module.to_string();
    let key_name = key.to_string();
    let value_str = value.to_string();
    
    thread::scope(|s| {
        s.spawn(|| {
            let container_file = format!("tree/{}.json", container_name);
            
            if !Path::new(&container_file).exists() {
                return "ERROR: Container does not exist".to_string();
            }
            
            let mut current_data = match fs::read_to_string(&container_file) {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(data) => data,
                    Err(_) => return "ERROR: Failed to parse container file".to_string(),
                },
                Err(_) => return "ERROR: Failed to read container file".to_string(),
            };
            
            if let Some(array) = current_data.as_array_mut() {
                for item in array {
                    if let Some(obj) = item.as_object_mut() {
                        if obj.get("id").and_then(|v| v.as_str()) == Some(&module_name) {
                            obj.insert(key_name.clone(), serde_json::Value::String(value_str.clone()));
                            
                            let formatted_data = match serde_json::to_string_pretty(&current_data) {
                                Ok(data) => data,
                                Err(_) => return "ERROR: Failed to format data".to_string(),
                            };
                            
                            if let Err(_) = fs::write(&container_file, formatted_data) {
                                return "ERROR: Failed to write container file".to_string();
                            }
                            
                            return format!("SET {} {}", key_name, value_str);
                        }
                    }
                }
            }
            
            "ERROR: Module not found".to_string()
        }).join().unwrap_or_else(|_| "ERROR: Thread panic".to_string())
    })
}

pub fn handle_get(container: &str, module: &str, key: &str) -> String {
    let manager = get_container_manager();
    let _lock = manager.get_container_lock(container);
    
    let container_name = container.to_string();
    let module_name = module.to_string();
    let key_name = key.to_string();
    
    thread::scope(|s| {
        s.spawn(|| {
            let container_file = format!("tree/{}.json", container_name);
            
            if !Path::new(&container_file).exists() {
                return "ERROR: Container does not exist".to_string();
            }
            
            let content = match fs::read_to_string(&container_file) {
                Ok(content) => content,
                Err(_) => return "ERROR: Failed to read container file".to_string(),
            };
            
            let data: serde_json::Value = match serde_json::from_str(&content) {
                Ok(data) => data,
                Err(_) => return "ERROR: Failed to parse container file".to_string(),
            };
            
            if let Some(array) = data.as_array() {
                for item in array {
                    if let Some(obj) = item.as_object() {
                        if obj.get("id").and_then(|v| v.as_str()) == Some(&module_name) {
                            if let Some(value) = obj.get(&key_name) {
                                return value.as_str().unwrap_or("").to_string();
                            }
                        }
                    }
                }
            }
            
            "ERROR: Key not found".to_string()
        }).join().unwrap_or_else(|_| "ERROR: Thread panic".to_string())
    })
}

pub fn handle_list_modules(container: &str) -> String {
    let manager = get_container_manager();
    let _lock = manager.get_container_lock(container);
    
    let container_name = container.to_string();
    
    thread::scope(|s| {
        s.spawn(|| {
            let container_file = format!("tree/{}.json", container_name);
            
            if !Path::new(&container_file).exists() {
                return "ERROR: Container does not exist".to_string();
            }
            
            let content = match fs::read_to_string(&container_file) {
                Ok(content) => content,
                Err(_) => return "ERROR: Failed to read container file".to_string(),
            };
            
            let data: serde_json::Value = match serde_json::from_str(&content) {
                Ok(data) => data,
                Err(_) => return "ERROR: Failed to parse container file".to_string(),
            };
            
            if let Some(array) = data.as_array() {
                let modules: Vec<String> = array
                    .iter()
                    .filter_map(|item| item.as_object())
                    .filter_map(|obj| obj.get("id"))
                    .filter_map(|id| id.as_str())
                    .map(|s| s.to_string())
                    .collect();
                
                modules.join(", ")
            } else {
                "ERROR: Invalid container format".to_string()
            }
        }).join().unwrap_or_else(|_| "ERROR: Thread panic".to_string())
    })
}

pub fn handle_list_keys(container: &str, module: &str) -> String {
    let manager = get_container_manager();
    let _lock = manager.get_container_lock(container);
    
    let container_name = container.to_string();
    let module_name = module.to_string();
    
    thread::scope(|s| {
        s.spawn(|| {
            let container_file = format!("tree/{}.json", container_name);
            
            if !Path::new(&container_file).exists() {
                return "ERROR: Container does not exist".to_string();
            }
            
            let content = match fs::read_to_string(&container_file) {
                Ok(content) => content,
                Err(_) => return "ERROR: Failed to read container file".to_string(),
            };
            
            let data: serde_json::Value = match serde_json::from_str(&content) {
                Ok(data) => data,
                Err(_) => return "ERROR: Failed to parse container file".to_string(),
            };
            
            if let Some(array) = data.as_array() {
                for item in array {
                    if let Some(obj) = item.as_object() {
                        if obj.get("id").and_then(|v| v.as_str()) == Some(&module_name) {
                            let keys: Vec<String> = obj
                                .keys()
                                .filter(|&k| k != "id")
                                .map(|k| k.to_string())
                                .collect();
                            
                            return keys.join(", ");
                        }
                    }
                }
            }
            
            "ERROR: Module not found".to_string()
        }).join().unwrap_or_else(|_| "ERROR: Thread panic".to_string())
    })
}

fn replace_placeholder(container: &mut serde_json::Value, replacement_value: &str) {
    match container {
        serde_json::Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (key, val) in obj {
                if key == "id" {
                    new_obj.insert(key.clone(), serde_json::Value::String(replacement_value.to_string()));
                } else {
                    let mut new_val = val.clone();
                    replace_placeholder(&mut new_val, replacement_value);
                    new_obj.insert(key.clone(), new_val);
                }
            }
            *container = serde_json::Value::Object(new_obj);
        }
        serde_json::Value::Array(arr) => {
            let new_arr: Vec<serde_json::Value> = arr.iter()
                .map(|v| {
                    let mut new_v = v.clone();
                    replace_placeholder(&mut new_v, replacement_value);
                    new_v
                })
                .collect();
            *container = serde_json::Value::Array(new_arr);
        }
        _ => {}
    }
}



 
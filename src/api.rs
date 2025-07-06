// Copyright (c) 2025, TheByteSlayer, Triangular
// Stores structured Data in JSON Files and makes it accessible over TCP, written in Rust.

use std::sync::OnceLock;
use std::sync::mpsc;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use crate::configuration::Config;
use crate::tree;

static API_MANAGER: OnceLock<ApiManager> = OnceLock::new();

struct ThreadPool {
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));
        
        // Spawn worker threads without storing their handles
        for _ in 0..size {
            let receiver = std::sync::Arc::clone(&receiver);
            thread::spawn(move || loop {
                let job: Job = receiver.lock().unwrap().recv().unwrap();
                job();
            });
        }
        
        ThreadPool { sender }
    }
    
    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

pub struct ApiManager {
    thread_pool: ThreadPool,
}

impl ApiManager {
    pub fn new() -> Self {
        let thread_pool_size = num_cpus::get();
        let thread_pool = ThreadPool::new(thread_pool_size);
        Self {
            thread_pool,
        }
    }

    fn handle_connection(mut stream: TcpStream, silent: bool) {
        let mut buffer = [0; 1024];
        
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let request = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                    
                    if request.is_empty() {
                        continue;
                    }
                    
                    let response = process_request(&request);
                    
                    if let Err(e) = stream.write_all(response.as_bytes()) {
                        if !silent {
                            eprintln!("Failed to write response: {}", e);
                        }
                        break;
                    }
                }
                Err(e) => {
                    if !silent {
                        eprintln!("Error reading from stream: {}", e);
                    }
                    break;
                }
            }
        }
    }
}

pub fn get_api_manager() -> &'static ApiManager {
    API_MANAGER.get_or_init(|| ApiManager::new())
}

pub fn start_server(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let manager = get_api_manager();
    let listener = TcpListener::bind(config.address())?;
    
    if !config.silent {
        println!("Triangular Database listening on {}", config.address());
    }
    
    let silent = config.silent;
    
    for stream in listener.incoming() {
        let stream = stream?;
        
        manager.thread_pool.execute(move || {
            ApiManager::handle_connection(stream, silent);
        });
    }
    
    Ok(())
}

pub fn process_request(request: &str) -> String {
    let parts: Vec<&str> = request.split_whitespace().collect();
    
    if parts.is_empty() {
        return "ERROR: Empty request".to_string();
    }
    
    let command = parts[0].to_uppercase();
    
    match command.as_str() {
        "INIT" => {
            if parts.len() < 3 {
                return "ERROR: INIT requires container and value".to_string();
            }
            
            let container = parts[1];
            let value = parts[2];
            
            tree::handle_init(container, value)
        }
        "SET" => {
            if parts.len() < 5 {
                return "ERROR: SET requires container, module, key, and value".to_string();
            }
            
            let container = parts[1];
            let module = parts[2];
            let key = parts[3];
            let value = parts[4];
            
            tree::handle_set(container, module, key, value)
        }
        "GET" => {
            if parts.len() < 4 {
                return "ERROR: GET requires container, module, and key".to_string();
            }
            
            let container = parts[1];
            let module = parts[2];
            let key = parts[3];
            
            tree::handle_get(container, module, key)
        }
        "LIST" => {
            if parts.len() < 2 {
                return "ERROR: LIST requires container".to_string();
            }
            
            let container = parts[1];
            
            if parts.len() == 2 {
                tree::handle_list_modules(container)
            } else if parts.len() == 3 {
                let module = parts[2];
                tree::handle_list_keys(container, module)
            } else {
                "ERROR: LIST takes 1 or 2 arguments".to_string()
            }
        }
        _ => "ERROR: Unknown command".to_string(),
    }
} 
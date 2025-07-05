// Copyright (c) 2025, TheByteSlayer, Triangular
// Stores structured Data in JSON Files and makes it accessible over TCP, written in Rust.

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

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
    format!("Echo: {}", request.trim())
} 
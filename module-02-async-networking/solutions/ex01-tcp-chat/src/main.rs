mod client;
mod message;
mod room;
mod server;

use client::handle_client;
use server::ChatServer;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;

const DEFAULT_PORT: u16 = 8080;
const MAX_CONNECTIONS: usize = 1000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting TCP Chat Server...");

    // Create shared server state
    let server = Arc::new(ChatServer::new(MAX_CONNECTIONS));

    // Bind to TCP port
    let addr = format!("0.0.0.0:{}", DEFAULT_PORT);
    let listener = TcpListener::bind(&addr).await?;

    println!("Server listening on {}", addr);
    println!("Maximum connections: {}", MAX_CONNECTIONS);
    println!("Press Ctrl+C to shutdown");

    // Spawn accept loop
    let server_clone = Arc::clone(&server);
    let accept_task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((socket, _addr)) => {
                    let server = Arc::clone(&server_clone);
                    tokio::spawn(async move {
                        handle_client(socket, server).await;
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    // Wait for Ctrl+C
    match signal::ctrl_c().await {
        Ok(()) => {
            println!("\nShutdown signal received, stopping server...");
        }
        Err(err) => {
            eprintln!("Error listening for shutdown signal: {}", err);
        }
    }

    // Abort accept loop
    accept_task.abort();

    println!("Server stopped");
    Ok(())
}

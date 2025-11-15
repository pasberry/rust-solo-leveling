use crate::command::Command;
use crate::db::Db;
use crate::error::DbError;
use crate::resp::RespValue;
use bytes::BytesMut;
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

pub struct Server {
    listener: TcpListener,
    db: Db,
}

impl Server {
    pub async fn bind(addr: &str) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(addr).await?;
        let db = Db::new();

        // Spawn expiration background task
        db.clone().spawn_expiration_task();

        Ok(Server { listener, db })
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        info!("Redis clone server started");

        loop {
            let (socket, addr) = self.listener.accept().await?;
            info!("New connection from {}", addr);

            let db = self.db.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, db).await {
                    error!("Error handling connection from {}: {}", addr, e);
                }
                info!("Connection closed: {}", addr);
            });
        }
    }
}

async fn handle_connection(mut socket: TcpStream, db: Db) -> Result<(), std::io::Error> {
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        // Read data from socket
        let n = socket.read_buf(&mut buffer).await?;

        if n == 0 {
            // Connection closed
            return Ok(());
        }

        // Process all complete commands in the buffer
        while !buffer.is_empty() {
            let mut cursor = Cursor::new(&buffer[..]);

            match RespValue::parse(&mut cursor) {
                Ok(value) => {
                    let consumed = cursor.position() as usize;
                    debug!("Parsed RESP value: {:?}", value);

                    // Process command
                    let response = match process_command(value, &db).await {
                        Ok(resp) => resp,
                        Err(e) => {
                            warn!("Command error: {}", e);
                            RespValue::Error(e.to_string())
                        }
                    };

                    // Send response
                    let response_bytes = response.serialize();
                    socket.write_all(&response_bytes).await?;
                    socket.flush().await?;

                    // Remove consumed bytes from buffer
                    buffer.advance(consumed);
                }
                Err(crate::error::RespError::Incomplete) => {
                    // Need more data
                    break;
                }
                Err(e) => {
                    error!("RESP parse error: {}", e);
                    let error = RespValue::Error(format!("ERR Protocol error: {}", e));
                    socket.write_all(&error.serialize()).await?;
                    socket.flush().await?;
                    return Ok(());
                }
            }
        }
    }
}

async fn process_command(value: RespValue, db: &Db) -> Result<RespValue, DbError> {
    let command = Command::from_resp(value)?;
    debug!("Executing command: {:?}", command);
    command.execute(db).await
}

use bytes::Buf;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_server_ping() {
        let server = Server::bind("127.0.0.1:0").await.unwrap();
        let addr = server.listener.local_addr().unwrap();

        tokio::spawn(async move {
            server.run().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client = TcpStream::connect(addr).await.unwrap();

        // Send PING command
        let ping = RespValue::Array(Some(vec![RespValue::BulkString(Some(b"PING".to_vec()))]));
        client.write_all(&ping.serialize()).await.unwrap();
        client.flush().await.unwrap();

        // Read response
        let mut buffer = BytesMut::with_capacity(1024);
        client.read_buf(&mut buffer).await.unwrap();

        let mut cursor = Cursor::new(&buffer[..]);
        let response = RespValue::parse(&mut cursor).unwrap();

        assert_eq!(response, RespValue::SimpleString("PONG".to_string()));
    }

    #[tokio::test]
    async fn test_server_set_get() {
        let server = Server::bind("127.0.0.1:0").await.unwrap();
        let addr = server.listener.local_addr().unwrap();

        tokio::spawn(async move {
            server.run().await.unwrap();
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut client = TcpStream::connect(addr).await.unwrap();

        // Send SET command
        let set = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"SET".to_vec())),
            RespValue::BulkString(Some(b"mykey".to_vec())),
            RespValue::BulkString(Some(b"myvalue".to_vec())),
        ]));
        client.write_all(&set.serialize()).await.unwrap();
        client.flush().await.unwrap();

        // Read SET response
        let mut buffer = BytesMut::with_capacity(1024);
        client.read_buf(&mut buffer).await.unwrap();
        let mut cursor = Cursor::new(&buffer[..]);
        let response = RespValue::parse(&mut cursor).unwrap();
        assert_eq!(response, RespValue::SimpleString("OK".to_string()));

        // Send GET command
        buffer.clear();
        let get = RespValue::Array(Some(vec![
            RespValue::BulkString(Some(b"GET".to_vec())),
            RespValue::BulkString(Some(b"mykey".to_vec())),
        ]));
        client.write_all(&get.serialize()).await.unwrap();
        client.flush().await.unwrap();

        // Read GET response
        client.read_buf(&mut buffer).await.unwrap();
        let mut cursor = Cursor::new(&buffer[..]);
        let response = RespValue::parse(&mut cursor).unwrap();
        assert_eq!(
            response,
            RespValue::BulkString(Some(b"myvalue".to_vec()))
        );
    }
}

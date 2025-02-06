use crate::logger::{log, LogSeverity};
use crate::protocol::handshake::*;
use crate::protocol::packet::*;
use crate::protocol::status::StatusResponsePacket;
use serde_json::json;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn run() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();
    log("Listening on port 25565".to_string(), LogSeverity::Info);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        log(format!("New connection from: {}", addr), LogSeverity::Info);
        tokio::spawn(handle_connection(socket));
    }
}

async fn handle_connection(mut socket: tokio::net::TcpStream) {
    let mut buffer = [0u8; 1024];
    match socket.read(&mut buffer).await {
        Ok(n) if n > 0 => {
            log(format!("Received {} bytes", n), LogSeverity::Info);
            log(
                format!("Received packet: {:?}", &buffer[..n]),
                LogSeverity::Info,
            );

            // // Check for legacy ping (packet starting with 0xFE)
            // if buffer[0] == 0xFE {
            //     log("Legacy ping detected".to_string(), LogSeverity::Info);
            //     handle_legacy_ping(socket).await;
            //     return;
            // }

            let mut packet_buffer = MinecraftPacketBuffer::from_bytes(buffer[..n].to_vec());
            match HandshakePacket::read(&mut packet_buffer) {
                Ok(handshake) => {
                    log(
                        format!("Received handshake: {:?}", handshake),
                        LogSeverity::Info,
                    );
                    if let Err(e) = handle_handshake(socket, handshake).await {
                        log(
                            format!("Failed to handle handshake: {}", e),
                            LogSeverity::Error,
                        );
                    }
                }
                Err(e) => log(
                    format!("Failed to parse handshake: {}", e),
                    LogSeverity::Error,
                ),
            }
        }
        Ok(_) => {}
        Err(e) => log(
            format!("Failed to read from socket: {}", e),
            LogSeverity::Error,
        ),
    }
}

async fn handle_legacy_ping(mut socket: tokio::net::TcpStream) {
    // Implement proper legacy ping handling or simply shut down.
    let _ = socket.shutdown().await;
}

async fn handle_handshake(
    mut socket: tokio::net::TcpStream,
    handshake: HandshakePacket,
) -> io::Result<()> {
    match handshake.next_state {
        1 => {
            // Wait for the status request packet
            let mut buffer = [0u8; 1024];
            socket.read(&mut buffer).await?;

            // Build and send status response
            let status_json = json!({
                "version": {
                    "name": "1.16.5",
                    "protocol": 754
                },
                "players": {
                    "max": 100,
                    "online": 0,
                    "sample": []
                },
                "description": {
                    "text": "Elytra Server"
                }
            });
            let response = StatusResponsePacket {
                response: status_json.to_string(),
            };

            let mut packet_buffer = MinecraftPacketBuffer::new();
            response.write(&mut packet_buffer).unwrap();

            // Write length prefix and packet data
            let mut final_buffer = MinecraftPacketBuffer::new();
            final_buffer.write_varint(packet_buffer.buffer.len() as i32);
            final_buffer.buffer.extend_from_slice(&packet_buffer.buffer);

            socket.write_all(&final_buffer.buffer).await?;
        }
        2 => {
            // Handle login state
            let mut response = MinecraftPacketBuffer::new();
            response.write_varint(0x00);
            socket.write_all(&response.buffer).await?;
        }
        _ => {
            log(
                format!("Unknown next state: {}", handshake.next_state),
                LogSeverity::Error,
            );
        }
    }
    Ok(())
}

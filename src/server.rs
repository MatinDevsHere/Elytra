﻿use crate::logger::{log, LogSeverity};
use crate::protocol::handshake::*;
use crate::protocol::packet::*;
use crate::protocol::status::StatusResponsePacket;
use serde_json::json;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use LogSeverity::*;

/// Starts the server and listens for incoming connections.
/// The server will listen on port 25565 by default.
pub async fn run() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();
    log("Listening on port 25565".to_string(), Info);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        log(format!("New connection from: {}", addr), Info);
        tokio::spawn(handle_connection(socket));
    }
}

/// Handles incoming connections. Each connection is handled in a separate task.
/// Reads the packet from the socket and handles the handshake.
/// The handshake is handled in a separate function.
async fn handle_connection(mut socket: TcpStream) {
    let mut buffer = [0u8; 1024];
    match socket.read(&mut buffer).await {
        Ok(size) if size > 0 => {
            log(format!("Received {} bytes", size), Info);
            log(format!("Received packet: {:?}", &buffer[..size]), Info);

            let mut packet_buffer = MinecraftPacketBuffer::from_bytes(buffer[..size].to_vec());
            match HandshakePacket::read(&mut packet_buffer) {
                Ok(handshake) => {
                    log(format!("Received handshake: {:?}", handshake), Info);
                    if let Err(handshake_error) = handle_handshake(socket, handshake).await {
                        log(
                            format!("Failed to handle handshake: {}", handshake_error),
                            Error,
                        );
                    }
                }
                Err(handshake_parse_error) => log(
                    format!("Failed to parse handshake: {}", handshake_parse_error),
                    Error,
                ),
            }
        }
        Ok(_) => panic!("This should never happen"),
        Err(socket_read_error) => log(
            format!("Failed to read from socket: {}", socket_read_error),
            Error,
        ),
    }
}

/// Handles the handshake packet next state
async fn handle_handshake(mut socket: TcpStream, handshake: HandshakePacket) -> io::Result<()> {
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
                response_json: status_json.to_string(),
            };

            let mut packet_buffer = MinecraftPacketBuffer::new();
            response.write(&mut packet_buffer)?;

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
            panic!("Unknown next state: {}", handshake.next_state);
        }
    }
    Ok(())
}

use crate::logger::{log, LogSeverity};
use crate::protocol::handshake::*;
use crate::protocol::packet::*;
use crate::protocol::status::StatusResponsePacket;
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
            log(format!("Received {} bytes", size), Debug);
            log(format!("Received packet: {:?}", &buffer[..size]), Debug);

            let mut packet_buffer = MinecraftPacketBuffer::from_bytes(buffer[..size].to_vec());
            match HandshakePacket::read(&mut packet_buffer) {
                Ok(handshake) => {
                    log(format!("Received handshake: {:?}", handshake), Debug);
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
        // Status request
        1 => {
            let mut raw_buffer = [0u8; 1024];
            socket.read(&mut raw_buffer).await?;

            let response = StatusResponsePacket::new();
            let mut response_buffer = MinecraftPacketBuffer::new();
            response.write(&mut response_buffer)?;

            let mut packet_with_length = MinecraftPacketBuffer::new();
            packet_with_length.write_varint(response_buffer.buffer.len() as i32);
            packet_with_length
                .buffer
                .extend_from_slice(&response_buffer.buffer);

            socket.write_all(&packet_with_length.buffer).await?;
        }
        // Login request
        2 => {}
        _ => panic!("Unknown next state: {}", handshake.next_state),
    }
    Ok(())
}

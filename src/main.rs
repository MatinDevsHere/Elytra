mod logger;
mod protocol;

use logger::{log, LogSeverity::*};
use protocol::handshake::HandshakePacket;
use protocol::packet::MinecraftPacketBuffer;
use protocol::packet::Packet;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    log("Elytra init".to_string(), Info);

    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();
    log("Listening on port 25565".to_string(), Info);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        log(format!("New connection from: {}", addr), Info);

        tokio::spawn(async move {
            let mut buffer = [0; 1024];

            match socket.read(&mut buffer).await {
                Ok(size) => {
                    log(format!("Received {} bytes", size), Info);

                    let mut packet_buffer =
                        MinecraftPacketBuffer::from_bytes(buffer[..size].to_vec());

                    match HandshakePacket::read(&mut packet_buffer) {
                        Ok(handshake) => {
                            log(format!("Received handshake: {:?}", handshake), Info);

                            let mut response = MinecraftPacketBuffer::new();
                            response.write_varint(0x00);
                            if let Err(socket_write_error) =
                                socket.write_all(&response.buffer).await
                            {
                                log(
                                    format!("Failed to send response: {}", socket_write_error),
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
                Err(socket_read_error) => log(
                    format!("Failed to read from socket: {}", socket_read_error),
                    Error,
                ),
            }
        });
    }
}

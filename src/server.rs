use crate::logger::{log, LogSeverity};
use crate::protocol::handshake::*;
use crate::protocol::join_game::JoinGamePacket;
use crate::protocol::login::{LoginStartPacket, LoginSuccessPacket};
use crate::protocol::packet::*;
use crate::protocol::status::StatusResponsePacket;
use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use LogSeverity::*;

/// Starts the server and listens for incoming connections.
/// The server will listen on port 25565 by default.
pub async fn run() {
    // TODO: Should be an option for manually setting IP and Port
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

            let mut handshake_packet_buffer =
                MinecraftPacketBuffer::from_bytes(buffer[..size].to_vec());
            match HandshakePacket::read_from_buffer(&mut handshake_packet_buffer) {
                Ok(handshake_packet) => {
                    log(format!("Received handshake: {:?}", handshake_packet), Debug);
                    if let Err(handshake_error) =
                        handle_handshake_next_state(socket, handshake_packet).await
                    {
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
async fn handle_handshake_next_state(
    mut socket: TcpStream,
    handshake: HandshakePacket,
) -> io::Result<()> {
    let mut raw_buffer = [0u8; 1024];
    match handshake.next_state {
        // Status request
        1 => {
            socket.read(&mut raw_buffer).await?;

            let response = StatusResponsePacket::new();
            send_packet(response, &mut socket).await?;
        }
        // Login request
        2 => {
            socket.read(&mut raw_buffer).await?;

            let mut login_start_packet_buffer =
                MinecraftPacketBuffer::from_bytes(raw_buffer.to_vec());

            if let Ok(login_start) =
                LoginStartPacket::read_from_buffer(&mut login_start_packet_buffer)
            {
                log(
                    format!("Player {} attempting to login", login_start.username),
                    Debug,
                );

                let login_success_packet = LoginSuccessPacket::new(login_start.username);
                send_packet(login_success_packet, &mut socket).await?;

                let join_game_packet_buffer =
                    JoinGamePacket::new(1, vec!["wow".to_owned()], "main".to_owned());
                send_packet(join_game_packet_buffer, &mut socket).await?;
            }
        }
        _ => panic!("Unknown next state: {}", handshake.next_state),
    }
    Ok(())
}

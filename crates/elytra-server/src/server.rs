use elytra_logger::log::log;
use elytra_logger::severity::LogSeverity::{Debug, Error, Info};
use elytra_logger::systime;
use elytra_protocol::client_settings::ClientSettingsPacket;
use elytra_protocol::declare_commands::{CommandNode, DeclareCommandsPacket, Parser, StringType};
use elytra_protocol::declare_recipes::DeclareRecipesPacket;
use elytra_protocol::handshake::*;
use elytra_protocol::held_item_change::HeldItemChangePacket;
use elytra_protocol::join_game::JoinGamePacket;
use elytra_protocol::keep_alive::KeepAlivePacket;
use elytra_protocol::login::{LoginStartPacket, LoginSuccessPacket};
use elytra_protocol::packet::*;
use elytra_protocol::player_position_and_look::PlayerPositionAndLook;
use elytra_protocol::session::PlayerSession;
use elytra_protocol::session_manager::SessionManager;
use elytra_protocol::status::StatusResponsePacket;
use elytra_protocol::update_light::UpdateLightPacket;
use elytra_protocol::update_view_position::UpdateViewPositionPacket;
use once_cell::sync;
use std::sync::Arc;
use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, Instant};

// Global session manager
static SESSION_MANAGER: sync::Lazy<Arc<RwLock<SessionManager>>> =
    sync::Lazy::new(|| Arc::new(RwLock::new(SessionManager::new())));

/// Starts the server and listens for incoming connections.
/// The server will listen on port 25565 by default.
pub async fn run() {
    // TODO: Should be an option for manually setting IP and Port
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();
    log("Listening on port 25565".to_owned(), Info);

    // Spawn keep-alive checker task
    tokio::spawn(keep_alive_checker());

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        log(format!("New connection from: {}", addr), Info);
        tokio::spawn(handle_connection(socket));
    }
}

/// Task that checks for timed-out connections
async fn keep_alive_checker() {
    let mut interval = interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        let mut session_manager = SESSION_MANAGER.write().await;

        // Check for timed-out sessions
        let to_remove = session_manager.check_keep_alives().await;
        for username in to_remove {
            if let Some(session) = session_manager.remove_session(&username) {
                log(format!("Player {} timed out", session.username), Info);
            }
        }
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let mut buffer = [0u8; 1024];
    match socket.read(&mut buffer).await {
        Ok(size) if size > 0 => {
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
        Err(socket_read_error) => log(
            format!("Failed to read from socket: {}", socket_read_error),
            Error,
        ),
        Ok(_) => panic!("This should never happen"),
    }
}

/// Handles the play state after login and join game
async fn handle_play_state(socket: TcpStream, username: String) -> io::Result<()> {
    let mut raw_buffer = [0u8; 1024];
    let mut last_keep_alive_time = Instant::now();

    // Create session with split socket
    let (session, mut reader) = PlayerSession::new(username.clone(), socket);

    // Add session to manager
    {
        let mut session_manager = SESSION_MANAGER.write().await;
        session_manager.add_session(session);
    }

    loop {
        // Send keep-alive packet every 10 seconds
        if last_keep_alive_time.elapsed() >= Duration::from_secs(10) {
            let keep_alive_id = systime::unix_timestamp();
            let keep_alive_packet = KeepAlivePacket::new(keep_alive_id);

            {
                let mut session_manager = SESSION_MANAGER.write().await;
                if let Some(session) = session_manager.get_session(&username) {
                    session.last_keep_alive_id = keep_alive_id;
                    session.last_keep_alive_time = Instant::now();
                    session.send_packet(keep_alive_packet).await?;
                }
            }

            last_keep_alive_time = Instant::now();
        }

        match reader.read(&mut raw_buffer).await {
            Ok(size) if size > 0 => {
                let mut packet_buffer =
                    MinecraftPacketBuffer::from_bytes(raw_buffer[..size].to_vec());
                let packet_id = packet_buffer.read_varint()?;

                match packet_id {
                    // Keep-alive response
                    0x0F => {
                        if let Ok(keep_alive) =
                            KeepAlivePacket::read_from_buffer(&mut packet_buffer)
                        {
                            let mut session_manager = SESSION_MANAGER.write().await;
                            if let Some(session) = session_manager.get_session(&username) {
                                if keep_alive.keep_alive_id == session.last_keep_alive_id {
                                    session.last_keep_alive_response = Instant::now();
                                }
                            }

                            log(
                                format!("Received keep alive packet from player: {}", username),
                                Debug,
                            );
                        }
                    }
                    // Player Position
                    0x11 => {
                        let x = packet_buffer.read_f64()?;
                        let y = packet_buffer.read_f64()?;
                        let z = packet_buffer.read_f64()?;
                        let yaw = packet_buffer.read_f32()?;
                        let pitch = packet_buffer.read_f32()?;

                        let mut session_manager = SESSION_MANAGER.write().await;
                        if let Some(session) = session_manager.get_session(&username) {
                            session.update_position(x, y, z, yaw, pitch);
                            session_manager
                                .broadcast_position_updates(&username)
                                .await?;
                        }
                    }
                    // Client Settings packet
                    0x05 => {
                        if let Ok(settings) =
                            ClientSettingsPacket::read_from_buffer(&mut packet_buffer)
                        {
                            log(
                                format!(
                                    "Received packet 0x{:02x} (Client Settings): {:?}",
                                    packet_id, settings
                                ),
                                Debug,
                            );
                        }
                    }
                    _ => {
                        log(
                            format!("Received unknown packet 0x{:02x}", packet_id),
                            Debug,
                        );
                    }
                }
            }
            Ok(_) => break, // Connection closed
            Err(e) => {
                log(format!("Error reading from socket: {}", e), Error);
                break;
            }
        }
    }

    // Remove session when connection ends
    {
        let mut session_manager = SESSION_MANAGER.write().await;
        session_manager.remove_session(&username);
        log(format!("Player {} disconnected", username), Info);
    }

    Ok(())
}

/// Creates a command graph with basic commands
#[allow(dead_code)]
fn create_command_graph() -> DeclareCommandsPacket {
    let mut declare_commands_packet = DeclareCommandsPacket::new();

    // Add /help command
    let help_node = CommandNode::new_literal("help", true);
    let help_index = declare_commands_packet.add_node(help_node);

    // Add /gamemode command with argument
    let gamemode_node = CommandNode::new_literal("gamemode", false);
    let gamemode_index = declare_commands_packet.add_node(gamemode_node);

    // Add gamemode argument node (creative, survival, etc)
    let mut gamemode_arg_node =
        CommandNode::new_argument("mode", Parser::String(StringType::SingleWord), true);
    gamemode_arg_node.set_suggestions("minecraft:ask_server");
    let gamemode_arg_index = declare_commands_packet.add_node(gamemode_arg_node);

    // Add /tp command with target argument
    let tp_node = CommandNode::new_literal("tp", false);
    let tp_index = declare_commands_packet.add_node(tp_node);

    // Add target argument for tp command
    let mut tp_target_node = CommandNode::new_argument(
        "target",
        Parser::Entity {
            single: true,
            only_players: true,
        },
        true,
    );
    tp_target_node.set_suggestions("minecraft:ask_server");
    let tp_target_index = declare_commands_packet.add_node(tp_target_node);

    // Connect the nodes
    declare_commands_packet.get_root_mut().add_child(help_index);
    declare_commands_packet
        .get_root_mut()
        .add_child(gamemode_index);
    declare_commands_packet.get_root_mut().add_child(tp_index);

    if let Some(gamemode_node) = declare_commands_packet.get_node_mut(gamemode_index) {
        gamemode_node.add_child(gamemode_arg_index);
    }

    if let Some(tp_node) = declare_commands_packet.get_node_mut(tp_index) {
        tp_node.add_child(tp_target_index);
    }

    declare_commands_packet
}

/// Creates a default light array for a chunk section
fn create_default_light_array() -> Vec<u8> {
    // Create a 2048-byte array (16x16x16 nibbles)
    let mut light_array = vec![0; 2048];
    // Set all light levels to 15 (full brightness) for testing
    for i in 0..2048 {
        light_array[i] = 0xFF; // Both nibbles set to 15
    }
    light_array
}

/// Creates initial light data for the spawn chunk
fn create_spawn_chunk_light() -> io::Result<UpdateLightPacket> {
    // For initial spawn chunk at 0,0
    let chunk_x = 0;
    let chunk_z = 0;

    // Create light arrays for each section that needs light
    let mut sky_light_arrays = Vec::new();
    let mut block_light_arrays = Vec::new();

    // For testing, we'll add light data for sections -1 to 16 (18 sections total)
    for _ in 0..18 {
        sky_light_arrays.push(create_default_light_array());
        block_light_arrays.push(create_default_light_array());
    }

    // Create bitmasks - set all 18 bits to 1 to indicate we're sending all sections
    let light_mask = 0b111111111111111111; // 18 bits set to 1

    UpdateLightPacket::new(
        chunk_x,
        chunk_z,
        true, // trust edges
        light_mask,
        light_mask,
        0, // no empty sky light sections
        0, // no empty block light sections
        sky_light_arrays,
        block_light_arrays,
    )
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

                // TODO: Implement login checks

                let login_success_packet = LoginSuccessPacket::new(login_start.username.clone());
                send_packet(login_success_packet, &mut socket).await?;

                let join_game_packet = JoinGamePacket::new(
                    1,
                    vec!["minecraft:overworld".to_owned()],
                    "minecraft:overworld".to_owned(),
                );
                send_packet(join_game_packet, &mut socket).await?;

                let held_item_change_packet = HeldItemChangePacket::new(0);
                send_packet(held_item_change_packet, &mut socket).await?;

                let declare_recipes_packet = DeclareRecipesPacket::new();
                send_packet(declare_recipes_packet, &mut socket).await?;

                let declare_commands_packet = create_command_graph();
                send_packet(declare_commands_packet, &mut socket).await?;

                let update_view_position_packet = UpdateViewPositionPacket::new(0, 0);
                send_packet(update_view_position_packet, &mut socket).await?;

                // Send initial light data for spawn chunk
                if let Ok(light_packet) = create_spawn_chunk_light() {
                    send_packet(light_packet, &mut socket).await?;
                    log("Sent initial light data for spawn chunk".to_owned(), Debug);
                }

                // Send initial position and look
                let player_position = PlayerPositionAndLook::new(
                    0.0,  // x - spawn at origin
                    64.0, // y - reasonable spawn height
                    0.0,  // z - spawn at origin
                    0.0,  // yaw - looking straight ahead
                    0.0,  // pitch - looking straight ahead
                    0,    // flags - all values are absolute
                    0,    // teleport ID - first teleport
                );
                send_packet(player_position, &mut socket).await?;

                // After sending join game packet, transition to play state
                handle_play_state(socket, login_start.username).await?;
            }
        }
        _ => panic!("Unknown next state: {}", handshake.next_state),
    }
    Ok(())
}

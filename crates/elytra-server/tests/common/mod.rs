use elytra_protocol::handshake::HandshakePacket;
use elytra_protocol::packet::{MinecraftPacketBuffer, Packet};
use tokio::io::{self as io, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn connect_to_server() -> TcpStream {
    TcpStream::connect("127.0.0.1:25565").await.unwrap()
}

pub async fn send_handshake(client: &mut TcpStream, next_state: i32) -> io::Result<()> {
    let handshake = HandshakePacket {
        protocol_version: 754, // Minecraft 1.16.5
        server_address: "localhost".to_string(),
        server_port: 25565,
        next_state,
    };

    send_packet(client, handshake).await
}

pub async fn send_packet<T: Packet>(client: &mut TcpStream, packet: T) -> io::Result<()> {
    let mut buffer = MinecraftPacketBuffer::new();
    packet.write_to_buffer(&mut buffer)?;

    let mut packet_with_length = MinecraftPacketBuffer::new();
    packet_with_length.write_varint(buffer.buffer.len() as i32);
    packet_with_length.buffer.extend_from_slice(&buffer.buffer);

    client.write_all(&packet_with_length.buffer).await
}

pub async fn read_response(client: &mut TcpStream) -> io::Result<String> {
    let mut response_buffer = vec![0u8; 1024];
    let n = client.read(&mut response_buffer).await?;
    Ok(String::from_utf8_lossy(&response_buffer[..n]).to_string())
}

pub fn assert_response_contains_status_fields(response: &str) {
    assert!(
        response.contains("version"),
        "Response missing version field"
    );
    assert!(
        response.contains("players"),
        "Response missing players field"
    );
    assert!(
        response.contains("description"),
        "Response missing description field"
    );
}

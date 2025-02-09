mod common;

use common::*;
use elytra::protocol::login::LoginStartPacket;
use elytra::protocol::packet::Packet;
use elytra::protocol::status::StatusRequestPacket;
use futures::future::join_all;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_server_handshake_status() {
    let mut client = connect_to_server().await;

    // Send handshake and status request
    send_handshake(&mut client, 1).await.unwrap();
    send_packet(&mut client, StatusRequestPacket).await.unwrap();

    // Read and verify response
    let response = read_response(&mut client).await.unwrap();
    assert_response_contains_status_fields(&response);
}

#[tokio::test]
async fn test_server_handshake_login() {
    let mut client = connect_to_server().await;

    // Send handshake and login start
    send_handshake(&mut client, 2).await.unwrap();
    send_packet(
        &mut client,
        LoginStartPacket {
            username: "TestPlayer".to_string(),
        },
    )
    .await
    .unwrap();

    // Read response and verify disconnect message
    let response = read_response(&mut client).await.unwrap();
    assert!(response.contains("disconnect") || response.contains("Disconnect"));
}

#[tokio::test]
async fn test_server_invalid_handshake() {
    let mut client = connect_to_server().await;

    // Send invalid handshake (next_state = 3)
    send_handshake(&mut client, 3).await.unwrap();

    // Connection should be closed
    let result = read_response(&mut client).await;
    assert!(result.is_err() || result.unwrap().is_empty());
}

#[tokio::test]
async fn test_server_concurrent_connections() {
    // Create multiple concurrent connections
    let mut handles = Vec::new();
    for i in 0..5 {
        handles.push(tokio::spawn(async move {
            // Add small delay to avoid exact simultaneous connections
            sleep(Duration::from_millis(i * 100)).await;

            let mut client = connect_to_server().await;

            // Send handshake and status request
            send_handshake(&mut client, 1).await.unwrap();
            send_packet(&mut client, StatusRequestPacket).await.unwrap();

            // Read response
            read_response(&mut client).await.unwrap()
        }));
    }

    // Wait for all connections to complete
    let results = join_all(handles).await;

    // Verify all connections received valid responses
    for result in results {
        let response = result.unwrap();
        assert_response_contains_status_fields(&response);
    }
}

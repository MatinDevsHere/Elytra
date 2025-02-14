use crate::packet::Packet;
use crate::player_position_and_look::PlayerPositionAndLook;
use crate::session::PlayerSession;
use std::collections::{HashMap, HashSet};
use std::io;

pub struct SessionManager {
    sessions: HashMap<String, PlayerSession>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn add_session(&mut self, session: PlayerSession) {
        self.sessions.insert(session.username.clone(), session);
    }

    pub fn remove_session(&mut self, username: &str) -> Option<PlayerSession> {
        self.sessions.remove(username)
    }

    pub fn get_session(&mut self, username: &str) -> Option<&mut PlayerSession> {
        self.sessions.get_mut(username)
    }

    /// Broadcast a packet to all players except those specified
    pub async fn broadcast_packet_except<T: Packet + Clone>(
        &mut self,
        packet: T,
        excluded_players: &HashSet<String>,
    ) -> io::Result<()> {
        for (username, session) in self.sessions.iter_mut() {
            if !excluded_players.contains(username) {
                session.send_packet(packet.clone()).await?;
            }
        }
        Ok(())
    }

    /// Broadcast a packet only to specified players
    pub async fn broadcast_packet_only<T: Packet + Clone>(
        &mut self,
        packet: T,
        included_players: &HashSet<String>,
    ) -> io::Result<()> {
        for username in included_players {
            if let Some(session) = self.sessions.get_mut(username) {
                session.send_packet(packet.clone()).await?;
            }
        }
        Ok(())
    }

    /// Broadcast a packet to all players except one
    pub async fn broadcast_packet<T: Packet + Clone>(
        &mut self,
        packet: T,
        except_username: Option<&str>,
    ) -> io::Result<()> {
        if let Some(username) = except_username {
            let mut excluded = HashSet::new();
            excluded.insert(username.to_string());
            self.broadcast_packet_except(packet, &excluded).await
        } else {
            let empty_set = HashSet::new();
            self.broadcast_packet_except(packet, &empty_set).await
        }
    }

    /// Broadcast position updates to specific players
    pub async fn broadcast_position_updates_to(
        &mut self,
        source_username: &str,
        target_players: &HashSet<String>,
    ) -> io::Result<()> {
        if let Some(source_session) = self.sessions.get(source_username) {
            let (x, y, z) = source_session.position;
            let position_packet = PlayerPositionAndLook::new(
                x,
                y,
                z,
                source_session.yaw,
                source_session.pitch,
                0, // flags - absolute position
                0, // teleport ID
            );
            self.broadcast_packet_only(position_packet, target_players)
                .await?;
        }
        Ok(())
    }

    pub async fn broadcast_position_updates(&mut self, source_username: &str) -> io::Result<()> {
        if let Some(source_session) = self.sessions.get(source_username) {
            let (x, y, z) = source_session.position;
            let position_packet = PlayerPositionAndLook::new(
                x,
                y,
                z,
                source_session.yaw,
                source_session.pitch,
                0, // flags - absolute position
                0, // teleport ID
            );
            let mut excluded = HashSet::new();
            excluded.insert(source_username.to_string());
            self.broadcast_packet_except(position_packet, &excluded)
                .await?;
        }
        Ok(())
    }

    pub async fn check_keep_alives(&mut self) -> Vec<String> {
        let mut to_remove = Vec::new();

        for (username, session) in self.sessions.iter() {
            if session.has_timed_out() {
                to_remove.push(username.clone());
            }
        }

        to_remove
    }

    /// Get a set of all online players
    pub fn get_all_players(&self) -> HashSet<String> {
        self.sessions.keys().cloned().collect()
    }

    /// Get online player names
    pub fn get_player_names(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }
}

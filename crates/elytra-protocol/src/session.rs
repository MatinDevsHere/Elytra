use crate::packet::{send_packet, Packet};
use tokio::io;
use tokio::io::{BufWriter, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::time::{Duration, Instant};

pub struct PlayerSession {
    pub username: String,
    pub writer: BufWriter<WriteHalf<TcpStream>>,
    pub last_keep_alive_id: i64,
    pub last_keep_alive_time: Instant,
    pub last_keep_alive_response: Instant,
    pub position: (f64, f64, f64),
    pub yaw: f32,
    pub pitch: f32,
}

impl PlayerSession {
    pub fn new(username: String, socket: TcpStream) -> (Self, ReadHalf<TcpStream>) {
        let (read, write) = tokio::io::split(socket);
        (
            Self {
                username,
                writer: BufWriter::new(write),
                last_keep_alive_id: 0,
                last_keep_alive_time: Instant::now(),
                last_keep_alive_response: Instant::now(),
                position: (0.0, 64.0, 0.0),
                yaw: 0.0,
                pitch: 0.0,
            },
            read,
        )
    }

    pub async fn send_packet<T: Packet>(&mut self, packet: T) -> io::Result<()> {
        send_packet(packet, &mut self.writer).await
    }

    pub fn should_send_keep_alive(&self) -> bool {
        self.last_keep_alive_time.elapsed() >= Duration::from_secs(10)
    }

    pub fn has_timed_out(&self) -> bool {
        self.last_keep_alive_response.elapsed() >= Duration::from_secs(30)
    }

    pub fn update_position(&mut self, x: f64, y: f64, z: f64, yaw: f32, pitch: f32) {
        self.position = (x, y, z);
        self.yaw = yaw;
        self.pitch = pitch;
    }
}

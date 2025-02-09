use std::io::{self};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// Packet trait. Contains the packet ID and the functions to write and read the packet.
pub trait Packet {
    /// Packet ID
    fn packet_id() -> i32
    where
        Self: Sized,
    {
        0x00
    }

    /// Reads the packet from the buffer. Default implementation is used for server-only packets, as
    /// they don't need to be read from the buffer.
    fn read_from_buffer(_buffer: &mut MinecraftPacketBuffer) -> io::Result<Self>
    where
        Self: Sized,
    {
        unimplemented!("Client-bound packets don't need read")
    }

    /// Writes the packet to the buffer. Default implementation is used for client-only packets, as
    /// they don't need to be written to the buffer.
    fn write_to_buffer(&self, _buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
        unimplemented!("Server-bound packets don't need write")
    }
}

/// Sends a packet to the client
pub async fn send_packet<T: Packet>(packet: T, socket: &mut TcpStream) -> io::Result<()> {
    let mut response_buffer = MinecraftPacketBuffer::new();
    packet.write_to_buffer(&mut response_buffer)?;

    let mut packet_with_length = MinecraftPacketBuffer::new();
    packet_with_length.write_varint(response_buffer.buffer.len() as i32);
    packet_with_length
        .buffer
        .extend_from_slice(&response_buffer.buffer);

    socket.write_all(&packet_with_length.buffer).await?;

    Ok(())
}

/// Minecraft packet buffer. Contains the buffer and the cursor.
/// The cursor is used to keep track of the current position in the buffer.
/// The buffer is used to store the packet data.
#[derive(Debug)]
pub struct MinecraftPacketBuffer {
    pub buffer: Vec<u8>,
    cursor: usize,
}

/// Minecraft packet buffer impl.
impl MinecraftPacketBuffer {
    /// Creates a new Minecraft packet buffer.
    /// The buffer is initialized with a capacity of 1024 bytes.
    /// The cursor is initialized to 0.
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cursor: 0,
        }
    }

    /// Creates a new Minecraft packet buffer from a byte array.
    /// The buffer is initialized with the given byte array.
    /// The cursor is initialized to 0.
    /// The buffer is not copied, so the caller must ensure that the byte array is valid for
    /// the lifetime of the Minecraft packet buffer.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            buffer: bytes,
            cursor: 0,
        }
    }

    /// Returns the current cursor position for the buffer.
    pub fn peek_byte(&self) -> Option<u8> {
        if self.cursor < self.buffer.len() {
            Some(self.buffer[self.cursor])
        } else {
            None
        }
    }

    /// Writes a VarInt to the buffer.
    /// A VarInt is a variable-length integer. It is encoded using 7 bits per byte, with the most
    /// significant bit of each byte set to 1 unless it is the final byte in the encoded
    /// representation.
    /// The VarInt is written to the buffer in network (big-endian) order.
    pub fn write_varint(&mut self, mut value: i32) {
        while (value & !0x7F) != 0 {
            self.buffer.push(((value & 0x7F) as u8) | 0x80);
            value >>= 7;
        }
        self.buffer.push((value & 0x7F) as u8);
    }

    /// Reads a VarInt from the buffer
    /// A VarInt is a variable-length integer. It is encoded using 7 bits per byte, with the most
    /// significant bit of each byte set to 1 unless it is the final byte in the encoded
    /// representation.
    /// The VarInt is read from the buffer in network (big-endian) order.
    pub fn read_varint(&mut self) -> io::Result<i32> {
        let mut result = 0;
        let mut shift = 0;

        loop {
            if self.cursor >= self.buffer.len() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "EOF while reading VarInt",
                ));
            }

            let byte = self.buffer[self.cursor];
            self.cursor += 1;

            result |= ((byte & 0x7F) as i32) << shift;
            shift += 7;

            if (byte & 0x80) == 0 {
                break;
            }

            if shift >= 32 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
            }
        }

        Ok(result)
    }

    /// Writes a u16 to the buffer.
    /// The u16 is written to the buffer in network (big-endian) order.
    pub fn write_string(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.write_varint(bytes.len() as i32);
        self.buffer.extend_from_slice(bytes);
    }

    /// Reads a string from the buffer.
    /// The string is read from the buffer in network (big-endian) order.
    pub fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_varint()? as usize;
        if self.cursor + length > self.buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes to read the full string",
            ));
        }
        let bytes = &self.buffer[self.cursor..self.cursor + length];
        self.cursor += length;
        String::from_utf8(bytes.to_vec()).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to convert bytes to UTF-8 string",
            )
        })
    }

    /// Writes a UUID to the buffer.
    /// The UUID is written as two longs in big-endian order.
    pub fn write_uuid(&mut self, value: uuid::Uuid) {
        self.buffer.extend_from_slice(value.as_bytes());
    }

    /// Reads a UUID from the buffer.
    /// The UUID is read as two longs in big-endian order.
    pub fn read_uuid(&mut self) -> io::Result<uuid::Uuid> {
        if self.cursor + 16 > self.buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes to read UUID",
            ));
        }
        let bytes = &self.buffer[self.cursor..self.cursor + 16];
        self.cursor += 16;
        Ok(uuid::Uuid::from_slice(bytes).unwrap())
    }

    // Write an u16 in network (big-endian) order.
    pub fn write_u16(&mut self, value: u16) {
        self.buffer.push((value >> 8) as u8);
        self.buffer.push((value & 0xFF) as u8);
    }

    // Read an u16 in network (big-endian) order.
    pub fn read_u16(&mut self) -> io::Result<u16> {
        if self.cursor + 2 > self.buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes to read u16",
            ));
        }
        let hi = self.buffer[self.cursor] as u16;
        let lo = self.buffer[self.cursor + 1] as u16;
        self.cursor += 2;
        Ok((hi << 8) | lo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    use uuid::Uuid;

    // Helper struct for testing Packet trait
    struct TestPacket {
        value: i32,
    }

    impl Packet for TestPacket {
        fn packet_id() -> i32 {
            0x42
        }

        fn write_to_buffer(&self, buffer: &mut MinecraftPacketBuffer) -> io::Result<()> {
            buffer.write_varint(self.value);
            Ok(())
        }

        fn read_from_buffer(buffer: &mut MinecraftPacketBuffer) -> io::Result<Self> {
            Ok(TestPacket {
                value: buffer.read_varint()?,
            })
        }
    }

    #[test]
    fn test_packet_buffer_new() {
        let buffer = MinecraftPacketBuffer::new();
        assert!(buffer.buffer.is_empty());
        assert_eq!(buffer.cursor, 0);
    }

    #[test]
    fn test_packet_buffer_from_bytes() {
        let bytes = vec![1, 2, 3];
        let buffer = MinecraftPacketBuffer::from_bytes(bytes.clone());
        assert_eq!(buffer.buffer, bytes);
        assert_eq!(buffer.cursor, 0);
    }

    #[test]
    fn test_peek_byte() {
        let buffer = MinecraftPacketBuffer::from_bytes(vec![1, 2, 3]);
        assert_eq!(buffer.peek_byte(), Some(1));

        let empty_buffer = MinecraftPacketBuffer::new();
        assert_eq!(empty_buffer.peek_byte(), None);
    }

    #[test]
    fn test_varint() {
        let test_cases = vec![0, 1, 127, 128, 255, 2147483647, -1, -2147483648];

        for value in test_cases {
            let mut buffer = MinecraftPacketBuffer::new();
            buffer.write_varint(value);

            let mut read_buffer = MinecraftPacketBuffer::from_bytes(buffer.buffer);
            assert_eq!(read_buffer.read_varint().unwrap(), value);
        }
    }

    #[test]
    fn test_string() {
        let test_strings = vec![
            "",
            "Hello",
            "Hello, World!",
            "🦀",         // Test UTF-8
            "こんにちは", // Test UTF-8
        ];

        for string in test_strings {
            let mut buffer = MinecraftPacketBuffer::new();
            buffer.write_string(string);

            let mut read_buffer = MinecraftPacketBuffer::from_bytes(buffer.buffer);
            assert_eq!(read_buffer.read_string().unwrap(), string);
        }
    }

    #[test]
    fn test_uuid() {
        let uuid = Uuid::new_v3(&Uuid::NAMESPACE_DNS, "wow".as_ref());
        let mut buffer = MinecraftPacketBuffer::new();
        buffer.write_uuid(uuid);

        let mut read_buffer = MinecraftPacketBuffer::from_bytes(buffer.buffer);
        assert_eq!(read_buffer.read_uuid().unwrap(), uuid);
    }

    #[test]
    fn test_u16() {
        let test_values = vec![0, 1, 255, 256, 65535];

        for value in test_values {
            let mut buffer = MinecraftPacketBuffer::new();
            buffer.write_u16(value);

            let mut read_buffer = MinecraftPacketBuffer::from_bytes(buffer.buffer);
            assert_eq!(read_buffer.read_u16().unwrap(), value);
        }
    }

    #[test]
    fn test_string_error_handling() {
        // Test invalid UTF-8
        let mut buffer = MinecraftPacketBuffer::new();
        buffer.write_varint(1); // Length of 1
        buffer.buffer.push(0xFF); // Invalid UTF-8 byte

        let result = buffer.read_string();
        assert!(result.is_err());

        // Test string too long for buffer
        let mut buffer = MinecraftPacketBuffer::new();
        buffer.write_varint(100); // Claim string length of 100
        buffer.buffer.push(0x41); // But only write 1 byte

        let result = buffer.read_string();
        assert!(result.is_err());
    }

    #[test]
    fn test_varint_error_handling() {
        // Test VarInt too long
        let mut buffer = MinecraftPacketBuffer::new();
        // Write 5 bytes with continuation bit set
        for _ in 0..5 {
            buffer.buffer.push(0xFF);
        }

        let result = buffer.read_varint();
        assert!(result.is_err());

        // Test unexpected EOF
        let mut buffer = MinecraftPacketBuffer::new();
        buffer.buffer.push(0x80); // Continuation bit set but no more bytes

        let result = buffer.read_varint();
        assert!(result.is_err());
    }

    #[test]
    fn test_uuid_error_handling() {
        let mut buffer = MinecraftPacketBuffer::new();
        buffer.buffer.extend_from_slice(&[0; 8]); // Only 8 bytes instead of required 16

        let result = buffer.read_uuid();
        assert!(result.is_err());
    }

    #[test]
    fn test_u16_error_handling() {
        let mut buffer = MinecraftPacketBuffer::new();
        buffer.buffer.push(0x00); // Only 1 byte instead of required 2

        let result = buffer.read_u16();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_packet() {
        use tokio::net::{TcpListener, TcpStream};

        // Start a TCP server
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Client task
        let client_task = tokio::spawn(async move {
            let mut client = TcpStream::connect(addr).await.unwrap();
            let packet = TestPacket { value: 42 };
            send_packet(packet, &mut client).await.unwrap();
        });

        // Accept connection and verify received data
        let (mut server, _) = listener.accept().await.unwrap();
        let mut buf = vec![0; 1024];
        let n = server.read(&mut buf).await.unwrap();

        // Verify the packet format
        let mut buffer = MinecraftPacketBuffer::from_bytes(buf[..n].to_vec());
        let packet_length = buffer.read_varint().unwrap();
        assert!(packet_length > 0);

        // Wait for client to complete
        client_task.await.unwrap();
    }
}

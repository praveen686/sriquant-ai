//! Monoio-native WebSocket implementation
//!
//! High-performance WebSocket client built specifically for monoio runtime,
//! following high-performance principles:
//! - Single-threaded async with monoio
//! - Minimal allocations
//! - Nanosecond precision timing
//! - Zero-copy where possible

use crate::errors::{ExchangeError, Result};
use crate::http::TlsStream;
use sriquant_core::{PerfTimer, nanos};

use monoio::net::TcpStream;
use tracing::{debug, info};
use url::Url;
use base64::Engine;
use sha1::{Sha1, Digest};
use webpki_roots;

/// WebSocket opcode constants
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xa,
}

impl OpCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(OpCode::Continuation),
            0x1 => Some(OpCode::Text),
            0x2 => Some(OpCode::Binary),
            0x8 => Some(OpCode::Close),
            0x9 => Some(OpCode::Ping),
            0xa => Some(OpCode::Pong),
            _ => None,
        }
    }
}

/// WebSocket frame header
#[derive(Debug, Clone)]
pub struct FrameHeader {
    pub fin: bool,
    pub opcode: OpCode,
    pub mask: Option<[u8; 4]>,
    pub payload_len: u64,
}

/// WebSocket frame
#[derive(Debug, Clone)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

impl Frame {
    /// Create a new text frame
    pub fn text(data: String) -> Self {
        Self {
            header: FrameHeader {
                fin: true,
                opcode: OpCode::Text,
                mask: Some(Self::generate_mask()),
                payload_len: data.len() as u64,
            },
            payload: data.into_bytes(),
        }
    }

    /// Create a new binary frame
    pub fn binary(data: Vec<u8>) -> Self {
        Self {
            header: FrameHeader {
                fin: true,
                opcode: OpCode::Binary,
                mask: Some(Self::generate_mask()),
                payload_len: data.len() as u64,
            },
            payload: data,
        }
    }

    /// Create a ping frame
    pub fn ping(data: Vec<u8>) -> Self {
        Self {
            header: FrameHeader {
                fin: true,
                opcode: OpCode::Ping,
                mask: Some(Self::generate_mask()),
                payload_len: data.len() as u64,
            },
            payload: data,
        }
    }

    /// Create a pong frame
    pub fn pong(data: Vec<u8>) -> Self {
        Self {
            header: FrameHeader {
                fin: true,
                opcode: OpCode::Pong,
                mask: Some(Self::generate_mask()),
                payload_len: data.len() as u64,
            },
            payload: data,
        }
    }

    /// Create a close frame
    pub fn close(code: u16, reason: String) -> Self {
        let mut payload = Vec::with_capacity(2 + reason.len());
        payload.extend_from_slice(&code.to_be_bytes());
        payload.extend_from_slice(reason.as_bytes());

        Self {
            header: FrameHeader {
                fin: true,
                opcode: OpCode::Close,
                mask: Some(Self::generate_mask()),
                payload_len: payload.len() as u64,
            },
            payload,
        }
    }

    /// Generate a random mask for client frames
    fn generate_mask() -> [u8; 4] {
        // Simple mask generation - in production, use proper RNG
        let timestamp = nanos();
        [
            (timestamp & 0xff) as u8,
            ((timestamp >> 8) & 0xff) as u8,
            ((timestamp >> 16) & 0xff) as u8,
            ((timestamp >> 24) & 0xff) as u8,
        ]
    }

    /// Apply mask to payload
    fn apply_mask(payload: &mut [u8], mask: &[u8; 4]) {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
    }

    /// Serialize frame to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut frame = Vec::new();

        // First byte: FIN + RSV + Opcode
        let first_byte = if self.header.fin { 0x80 } else { 0x00 } | (self.header.opcode as u8);
        frame.push(first_byte);

        // Second byte: MASK + Payload length
        let mask_bit = if self.header.mask.is_some() { 0x80 } else { 0x00 };
        
        if self.header.payload_len < 126 {
            frame.push(mask_bit | (self.header.payload_len as u8));
        } else if self.header.payload_len < 65536 {
            frame.push(mask_bit | 126);
            frame.extend_from_slice(&(self.header.payload_len as u16).to_be_bytes());
        } else {
            frame.push(mask_bit | 127);
            frame.extend_from_slice(&self.header.payload_len.to_be_bytes());
        }

        // Mask
        if let Some(mask) = self.header.mask {
            frame.extend_from_slice(&mask);
        }

        // Payload (masked if client)
        let mut payload = self.payload.clone();
        if let Some(mask) = &self.header.mask {
            Self::apply_mask(&mut payload, mask);
        }
        frame.extend_from_slice(&payload);

        frame
    }

    /// Parse frame from bytes
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize)> {
        if data.len() < 2 {
            return Err(ExchangeError::InvalidResponse("Insufficient data for WebSocket frame".to_string()));
        }

        let timer = PerfTimer::start("websocket_frame_parse".to_string());

        let first_byte = data[0];
        let second_byte = data[1];

        let fin = (first_byte & 0x80) != 0;
        let opcode = OpCode::from_u8(first_byte & 0x0f)
            .ok_or_else(|| ExchangeError::InvalidResponse("Invalid WebSocket opcode".to_string()))?;

        let masked = (second_byte & 0x80) != 0;
        let payload_len_initial = second_byte & 0x7f;

        let mut offset = 2;
        let payload_len = if payload_len_initial < 126 {
            payload_len_initial as u64
        } else if payload_len_initial == 126 {
            if data.len() < offset + 2 {
                return Err(ExchangeError::InvalidResponse("Insufficient data for extended payload length".to_string()));
            }
            let len = u16::from_be_bytes([data[offset], data[offset + 1]]) as u64;
            offset += 2;
            len
        } else {
            if data.len() < offset + 8 {
                return Err(ExchangeError::InvalidResponse("Insufficient data for extended payload length".to_string()));
            }
            let len = u64::from_be_bytes([
                data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
                data[offset + 4], data[offset + 5], data[offset + 6], data[offset + 7],
            ]);
            offset += 8;
            len
        };

        let mask = if masked {
            if data.len() < offset + 4 {
                return Err(ExchangeError::InvalidResponse("Insufficient data for mask".to_string()));
            }
            let mask = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
            offset += 4;
            Some(mask)
        } else {
            None
        };

        if data.len() < offset + payload_len as usize {
            return Err(ExchangeError::InvalidResponse("Insufficient data for payload".to_string()));
        }

        let mut payload = data[offset..offset + payload_len as usize].to_vec();
        if let Some(mask) = &mask {
            Self::apply_mask(&mut payload, mask);
        }

        let frame = Frame {
            header: FrameHeader {
                fin,
                opcode,
                mask,
                payload_len,
            },
            payload,
        };

        timer.log_elapsed();
        Ok((frame, offset + payload_len as usize))
    }
}

/// Monoio-native WebSocket client
pub struct MonoioWebSocket {
    stream: TlsStream,
    url: Url,
    connected: bool,
    close_sent: bool,
    buffer: Vec<u8>,
}

impl MonoioWebSocket {
    /// Create a new WebSocket connection
    pub async fn connect(url: Url) -> Result<Self> {
        let timer = PerfTimer::start("websocket_connect".to_string());
        
        info!("🔗 Connecting to WebSocket: {}", url);

        // Extract host and port
        let host = url.host_str()
            .ok_or_else(|| ExchangeError::InvalidUrl("No host in WebSocket URL".to_string()))?;
        let port = url.port().unwrap_or(443);

        // Establish TCP connection
        let tcp_stream = TcpStream::connect(&format!("{host}:{port}"))
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("TCP connection failed: {e}")))?;

        debug!("✅ TCP connection established to {}:{}", host, port);

        // Set up TLS configuration
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(
            webpki_roots::TLS_SERVER_ROOTS
                .iter()
                .cloned()
        );

        let config = std::sync::Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth()
        );

        // Create TLS connection
        let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
            .map_err(|e| ExchangeError::NetworkError(format!("Invalid server name: {e}")))?;
        let tls_conn = rustls::ClientConnection::new(config, server_name)
            .map_err(|e| ExchangeError::NetworkError(format!("TLS connection setup failed: {e}")))?;

        // Create TLS stream
        let mut tls_stream = TlsStream::new(tcp_stream, tls_conn);
        tls_stream.complete_handshake().await
            .map_err(|e| ExchangeError::NetworkError(format!("TLS handshake failed: {e}")))?;

        debug!("✅ TLS handshake completed");

        let mut websocket = Self {
            stream: tls_stream,
            url: url.clone(),
            connected: false,
            close_sent: false,
            buffer: Vec::with_capacity(8192),
        };

        // Perform WebSocket handshake
        websocket.perform_handshake().await?;

        timer.log_elapsed();
        info!("✅ WebSocket connection established to {}", url);

        Ok(websocket)
    }

    /// Perform WebSocket handshake
    async fn perform_handshake(&mut self) -> Result<()> {
        let timer = PerfTimer::start("websocket_handshake".to_string());

        // Generate WebSocket key
        let ws_key = self.generate_websocket_key();

        // Build handshake request
        let path = if self.url.path().is_empty() { "/" } else { self.url.path() };
        let query = self.url.query().map(|q| format!("?{q}")).unwrap_or_default();
        let host = self.url.host_str().unwrap();

        let handshake_request = format!(
            "GET {path}{query} HTTP/1.1\r\n\
             Host: {host}\r\n\
             Upgrade: websocket\r\n\
             Connection: Upgrade\r\n\
             Sec-WebSocket-Key: {ws_key}\r\n\
             Sec-WebSocket-Version: 13\r\n\
             \r\n"
        );

        debug!("Sending WebSocket handshake request");

        // Send handshake request
        self.stream.write_all(handshake_request.as_bytes()).await
            .map_err(|e| ExchangeError::NetworkError(format!("Failed to send handshake: {e}")))?;

        // Read handshake response
        let mut response_buffer = vec![0u8; 4096];
        let bytes_read = self.stream.read(&mut response_buffer).await
            .map_err(|e| ExchangeError::NetworkError(format!("Failed to read handshake response: {e}")))?;

        let response = String::from_utf8_lossy(&response_buffer[..bytes_read]);
        debug!("Received handshake response: {}", response);

        // Validate handshake response
        self.validate_handshake_response(&response, &ws_key)?;

        self.connected = true;
        timer.log_elapsed();
        debug!("✅ WebSocket handshake completed successfully");

        Ok(())
    }

    /// Generate WebSocket key for handshake
    fn generate_websocket_key(&self) -> String {
        let timestamp = nanos();
        let key_bytes = timestamp.to_be_bytes();
        base64::engine::general_purpose::STANDARD.encode(key_bytes)
    }

    /// Validate WebSocket handshake response
    fn validate_handshake_response(&self, response: &str, ws_key: &str) -> Result<()> {
        if !response.starts_with("HTTP/1.1 101") {
            return Err(ExchangeError::NetworkError("WebSocket handshake failed: invalid response code".to_string()));
        }

        let expected_accept = self.calculate_accept_key(ws_key);
        let accept_header = format!("sec-websocket-accept: {expected_accept}");

        if !response.to_lowercase().contains(&accept_header.to_lowercase()) {
            return Err(ExchangeError::NetworkError("WebSocket handshake failed: invalid accept key".to_string()));
        }

        Ok(())
    }

    /// Calculate WebSocket accept key
    fn calculate_accept_key(&self, ws_key: &str) -> String {
        const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        let combined = format!("{ws_key}{WS_GUID}");
        let mut hasher = Sha1::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        base64::engine::general_purpose::STANDARD.encode(hash)
    }

    /// Send a frame
    pub async fn send_frame(&mut self, frame: Frame) -> Result<()> {
        if !self.connected || self.close_sent {
            return Err(ExchangeError::NetworkError("WebSocket not connected".to_string()));
        }

        let timer = PerfTimer::start("websocket_send_frame".to_string());
        let frame_bytes = frame.to_bytes();

        debug!("Sending WebSocket frame: {:?} ({} bytes)", frame.header.opcode, frame_bytes.len());

        self.stream.write_all(&frame_bytes).await
            .map_err(|e| ExchangeError::NetworkError(format!("Failed to send frame: {e}")))?;

        if matches!(frame.header.opcode, OpCode::Close) {
            self.close_sent = true;
        }

        timer.log_elapsed();
        Ok(())
    }

    /// Send text message
    pub async fn send_text(&mut self, message: String) -> Result<()> {
        let frame = Frame::text(message);
        self.send_frame(frame).await
    }

    /// Send binary message
    pub async fn send_binary(&mut self, data: Vec<u8>) -> Result<()> {
        let frame = Frame::binary(data);
        self.send_frame(frame).await
    }

    /// Send ping
    pub async fn ping(&mut self, data: Vec<u8>) -> Result<()> {
        let frame = Frame::ping(data);
        self.send_frame(frame).await
    }

    /// Send pong  
    pub async fn pong(&mut self, data: Vec<u8>) -> Result<()> {
        let frame = Frame::pong(data);
        self.send_frame(frame).await
    }

    /// Receive next frame
    pub async fn receive_frame(&mut self) -> Result<Frame> {
        if !self.connected {
            return Err(ExchangeError::NetworkError("WebSocket not connected".to_string()));
        }

        let timer = PerfTimer::start("websocket_receive_frame".to_string());

        loop {
            // Try to parse a frame from the buffer
            if let Ok((frame, consumed)) = Frame::from_bytes(&self.buffer) {
                self.buffer.drain(..consumed);
                timer.log_elapsed();

                // Handle control frames automatically
                match frame.header.opcode {
                    OpCode::Ping => {
                        debug!("Received ping, sending pong");
                        self.pong(frame.payload.clone()).await?;
                        continue; // Continue reading for next frame
                    }
                    OpCode::Close => {
                        debug!("Received close frame");
                        if !self.close_sent {
                            let close_frame = Frame::close(1000, "Normal closure".to_string());
                            let _ = self.send_frame(close_frame).await;
                        }
                        self.connected = false;
                        return Ok(frame);
                    }
                    _ => return Ok(frame),
                }
            }

            // Need more data
            let mut temp_buffer = vec![0u8; 4096];
            let bytes_read = self.stream.read(&mut temp_buffer).await
                .map_err(|e| ExchangeError::NetworkError(format!("Failed to read frame: {e}")))?;

            if bytes_read == 0 {
                return Err(ExchangeError::NetworkError("WebSocket connection closed by peer".to_string()));
            }

            self.buffer.extend_from_slice(&temp_buffer[..bytes_read]);
        }
    }

    /// Receive next text message
    pub async fn receive_text(&mut self) -> Result<String> {
        let frame = self.receive_frame().await?;
        match frame.header.opcode {
            OpCode::Text => {
                String::from_utf8(frame.payload)
                    .map_err(|e| ExchangeError::InvalidResponse(format!("Invalid UTF-8 in text frame: {e}")))
            }
            _ => Err(ExchangeError::InvalidResponse("Expected text frame".to_string())),
        }
    }

    /// Close the WebSocket connection
    pub async fn close(&mut self, code: u16, reason: String) -> Result<()> {
        if !self.connected || self.close_sent {
            return Ok(());
        }

        info!("🔌 Closing WebSocket connection");
        let close_frame = Frame::close(code, reason);
        self.send_frame(close_frame).await?;
        
        // Try to wait for close response (best effort)
        match self.receive_frame().await {
            Ok(frame) if matches!(frame.header.opcode, OpCode::Close) => {
                debug!("✅ Close handshake completed");
            }
            Ok(_) => {
                debug!("Received non-close frame during close handshake");
            }
            Err(_) => {
                // Connection already closed or error - this is normal during close
                debug!("Connection terminated during close handshake (normal)");
            }
        }

        self.connected = false;
        Ok(())
    }

    /// Check if WebSocket is connected
    pub fn is_connected(&self) -> bool {
        self.connected && !self.close_sent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let frame = Frame::text("Hello WebSocket".to_string());
        assert_eq!(frame.header.opcode, OpCode::Text);
        assert_eq!(frame.payload, b"Hello WebSocket");
        assert!(frame.header.mask.is_some());
    }

    #[test]
    fn test_frame_serialization() {
        let frame = Frame::text("Hi".to_string());
        let bytes = frame.to_bytes();
        assert!(!bytes.is_empty());
        assert_eq!(bytes[0] & 0x0f, OpCode::Text as u8); // Check opcode
        assert!(bytes[1] & 0x80 != 0); // Check mask bit
    }

    #[test]
    fn test_websocket_key_generation() {
        // Create a fake/mock websocket for testing key generation only
        // We'll only test the key generation logic here
        let timestamp = nanos();
        let key_bytes = timestamp.to_be_bytes();
        let key = base64::engine::general_purpose::STANDARD.encode(key_bytes);
        
        assert!(!key.is_empty());
        
        // Should be base64 encoded
        assert!(base64::engine::general_purpose::STANDARD.decode(&key).is_ok());
    }

    #[test]
    fn test_accept_key_calculation() {
        // Test WebSocket accept key calculation directly without creating a WebSocket instance
        const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        let test_key = "dGhlIHNhbXBsZSBub25jZQ==";
        let combined = format!("{test_key}{WS_GUID}");
        let mut hasher = Sha1::new();
        hasher.update(combined.as_bytes());
        let hash = hasher.finalize();
        let accept_key = base64::engine::general_purpose::STANDARD.encode(hash);
        
        assert_eq!(accept_key, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }
}
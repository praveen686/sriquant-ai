//! Monoio-native HTTP/HTTPS client implementation
//!
//! High-performance architecture:
//! - Single-threaded async with monoio
//! - Direct TLS integration with rustls
//! - High-performance HTTP/1.1 implementation
//! - Zero-copy operations where possible

use crate::errors::{ExchangeError, Result};
use monoio::io::{AsyncReadRent, AsyncWriteRentExt};
use std::io::{Read, Write};
use monoio::net::TcpStream;
use rustls::{ClientConfig, ClientConnection};
use rustls::pki_types::ServerName;
use std::sync::Arc;
use webpki_roots;

/// Monoio-native HTTPS client
pub struct MonoioHttpsClient {
    tls_config: Arc<ClientConfig>,
}

/// HTTP response
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

/// TLS stream wrapper for monoio
pub struct TlsStream {
    stream: TcpStream,
    tls_conn: ClientConnection,
    write_buf: Vec<u8>,
    tls_read_buf: Vec<u8>,
    handshake_complete: bool,
}

impl MonoioHttpsClient {
    /// Create a new HTTPS client with default TLS configuration
    pub fn new() -> Result<Self> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.extend(
            webpki_roots::TLS_SERVER_ROOTS
                .iter()
                .cloned()
        );

        let tls_config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            tls_config: Arc::new(tls_config),
        })
    }

    /// Make an HTTPS GET request
    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.request("GET", url, None).await
    }

    /// Make an HTTPS POST request
    pub async fn post(&self, url: &str, body: Option<&str>) -> Result<HttpResponse> {
        self.request("POST", url, body).await
    }

    /// Make an HTTPS request
    pub async fn request(&self, method: &str, url: &str, body: Option<&str>) -> Result<HttpResponse> {
        let headers = std::collections::HashMap::new();
        self.request_with_headers(method, url, body, &headers).await
    }

    /// Make an HTTPS request with custom headers
    pub async fn request_with_headers(
        &self, 
        method: &str, 
        url: &str, 
        body: Option<&str>,
        headers: &std::collections::HashMap<&str, &str>
    ) -> Result<HttpResponse> {
        // Parse URL
        let parsed_url = url::Url::parse(url)
            .map_err(|e| ExchangeError::InvalidUrl(e.to_string()))?;
        
        let host = parsed_url.host_str()
            .ok_or_else(|| ExchangeError::InvalidUrl("No host in URL".to_string()))?;
        
        let port = parsed_url.port().unwrap_or(443);
        let path_and_query = if parsed_url.path().is_empty() { 
            "/".to_string() 
        } else {
            let mut path_and_query = parsed_url.path().to_string();
            if let Some(query) = parsed_url.query() {
                path_and_query.push('?');
                path_and_query.push_str(query);
            }
            path_and_query
        };
        
        // Connect to server
        let tcp_stream = TcpStream::connect(&format!("{host}:{port}"))
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("TCP connect failed: {e}")))?;

        // Establish TLS connection
        let server_name = ServerName::try_from(host.to_string())
            .map_err(|e| ExchangeError::NetworkError(format!("Invalid server name: {e:?}")))?;
        
        let tls_conn = ClientConnection::new(self.tls_config.clone(), server_name)
            .map_err(|e| ExchangeError::NetworkError(format!("TLS setup failed: {e}")))?;

        let mut tls_stream = TlsStream::new(tcp_stream, tls_conn);

        // Build HTTP request with custom headers
        let content_length = body.map(|b| b.len()).unwrap_or(0);
        let mut request = format!(
            "{method} {path_and_query} HTTP/1.1\r\n\
             Host: {host}\r\n\
             User-Agent: SriQuant.ai/1.0\r\n\
             Connection: close\r\n\
             Content-Length: {content_length}\r\n"
        );
        
        // Add custom headers
        for (key, value) in headers {
            request.push_str(&format!("{key}: {value}\r\n"));
        }
        
        
        // End headers and add body
        request.push_str("\r\n");
        if let Some(body) = body {
            request.push_str(body);
        }

        // Send request
        tls_stream.write_all(request.as_bytes()).await
            .map_err(|e| ExchangeError::NetworkError(format!("Write failed: {e}")))?;

        // Read response
        let response_data = tls_stream.read_to_end().await
            .map_err(|e| ExchangeError::NetworkError(format!("Read failed: {e}")))?;

        // Parse HTTP response
        self.parse_http_response(&response_data)
    }

    /// Parse HTTP response
    fn parse_http_response(&self, data: &[u8]) -> Result<HttpResponse> {
        let response_str = String::from_utf8_lossy(data);
        
        // Find the end of headers (double CRLF)
        let header_end = response_str.find("\r\n\r\n")
            .ok_or_else(|| ExchangeError::NetworkError("Invalid HTTP response: no header terminator".to_string()))?;
        
        let header_part = &response_str[..header_end];
        let body_part = &response_str[header_end + 4..]; // Skip the \r\n\r\n
        
        let mut lines = header_part.lines();
        
        // Parse status line
        let status_line = lines.next()
            .ok_or_else(|| ExchangeError::NetworkError("Empty response".to_string()))?;
        
        let status = status_line.split_whitespace()
            .nth(1)
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or_else(|| ExchangeError::NetworkError("Invalid status line".to_string()))?;

        // Parse headers
        let mut headers = Vec::new();
        
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        Ok(HttpResponse {
            status,
            headers,
            body: body_part.to_string(),
        })
    }
}

impl TlsStream {
    pub fn new(stream: TcpStream, tls_conn: ClientConnection) -> Self {
        Self {
            stream,
            tls_conn,
            write_buf: Vec::with_capacity(8192),
            tls_read_buf: Vec::with_capacity(8192),
            handshake_complete: false,
        }
    }

    /// Complete TLS handshake
    pub async fn complete_handshake(&mut self) -> Result<()> {
        if self.handshake_complete {
            return Ok(());
        }

        // TLS handshake loop
        loop {
            // Check if we need to send data to the server
            while self.tls_conn.wants_write() {
                // Clear and prepare write buffer
                self.write_buf.clear();
                self.write_buf.reserve(8192);
                
                // Write TLS data to buffer
                let tls_bytes = self.tls_conn.write_tls(&mut self.write_buf)
                    .map_err(|e| ExchangeError::NetworkError(format!("TLS write failed: {e}")))?;
                
                if tls_bytes > 0 {
                    let (result, _) = self.stream.write_all(self.write_buf.clone()).await;
                    result.map_err(|e| ExchangeError::NetworkError(format!("TCP write failed: {e}")))?;
                }
            }

            // Check if handshake is complete
            if !self.tls_conn.is_handshaking() {
                self.handshake_complete = true;
                break;
            }

            // Read data from server if needed
            if self.tls_conn.wants_read() {
                let buffer = vec![0u8; 4096];
                let (result, buf) = self.stream.read(buffer).await;
                let bytes_read = result.map_err(|e| ExchangeError::NetworkError(format!("TCP read failed: {e}")))?;
                
                if bytes_read == 0 {
                    return Err(ExchangeError::NetworkError("Connection closed during handshake".to_string()));
                }

                // Process received TLS data
                self.tls_conn.read_tls(&mut std::io::Cursor::new(&buf[..bytes_read]))
                    .map_err(|e| ExchangeError::NetworkError(format!("TLS read failed: {e}")))?;
                
                // Process any TLS messages
                self.tls_conn.process_new_packets()
                    .map_err(|e| ExchangeError::NetworkError(format!("TLS process failed: {e}")))?;
            } else {
                // If TLS doesn't want to read or write, but handshake is not complete, something is wrong
                if !self.tls_conn.wants_write() {
                    return Err(ExchangeError::NetworkError("TLS handshake stalled".to_string()));
                }
            }
        }

        Ok(())
    }

    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        // Ensure handshake is complete
        self.complete_handshake().await?;

        // Write application data through TLS
        self.tls_conn.writer().write_all(data)
            .map_err(|e| ExchangeError::NetworkError(format!("TLS application write failed: {e}")))?;

        // Send any pending TLS data
        while self.tls_conn.wants_write() {
            self.write_buf.clear();
            self.write_buf.reserve(8192);
            
            let tls_bytes = self.tls_conn.write_tls(&mut self.write_buf)
                .map_err(|e| ExchangeError::NetworkError(format!("TLS write failed: {e}")))?;
            
            if tls_bytes > 0 {
                let (result, _) = self.stream.write_all(self.write_buf.clone()).await;
                result.map_err(|e| ExchangeError::NetworkError(format!("TCP write failed: {e}")))?;
            }
        }

        Ok(())
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // Ensure handshake is complete
        self.complete_handshake().await?;

        // First try to read any available decrypted data
        match self.tls_conn.reader().read(buf) {
            Ok(n) if n > 0 => return Ok(n),
            Ok(_) => {}, // No decrypted data available
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {},
            Err(e) => return Err(ExchangeError::NetworkError(format!("TLS read failed: {e}"))),
        }

        // Need to read more encrypted data from TCP
        let tcp_buffer = vec![0u8; 4096];
        let (result, tcp_buf) = self.stream.read(tcp_buffer).await;
        let bytes_read = result.map_err(|e| ExchangeError::NetworkError(format!("TCP read failed: {e}")))?;

        if bytes_read == 0 {
            return Ok(0); // Connection closed
        }

        // Process the encrypted data through TLS
        self.tls_conn.read_tls(&mut std::io::Cursor::new(&tcp_buf[..bytes_read]))
            .map_err(|e| ExchangeError::NetworkError(format!("TLS read_tls failed: {e}")))?;

        let _tls_state = self.tls_conn.process_new_packets()
            .map_err(|e| ExchangeError::NetworkError(format!("TLS process_new_packets failed: {e}")))?;

        // Try reading decrypted data again after processing TLS packets
        match self.tls_conn.reader().read(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(ExchangeError::NetworkError(format!("TLS read failed: {e}"))),
        }
    }

    async fn read_to_end(&mut self) -> Result<Vec<u8>> {
        // Ensure handshake is complete
        self.complete_handshake().await?;

        let mut response_data = Vec::new();
        let mut tcp_buffer = vec![0u8; 4096];

        loop {
            // First check if we have decrypted data available
            self.tls_read_buf.clear();
            self.tls_read_buf.resize(4096, 0);
            
            match self.tls_conn.reader().read(&mut self.tls_read_buf) {
                Ok(0) => {
                    // No more decrypted data available, need to read more from TCP
                },
                Ok(n) => {
                    response_data.extend_from_slice(&self.tls_read_buf[..n]);
                    continue;
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No more decrypted data available right now
                },
                Err(e) => {
                    return Err(ExchangeError::NetworkError(format!("TLS read failed: {e}")));
                }
            }

            // Read more encrypted data from TCP
            let (result, buf) = self.stream.read(tcp_buffer).await;
            let bytes_read = result.map_err(|e| ExchangeError::NetworkError(format!("TCP read failed: {e}")))?;
            
            if bytes_read == 0 {
                break; // Connection closed
            }

            // Process received TLS data
            let mut cursor = std::io::Cursor::new(&buf[..bytes_read]);
            self.tls_conn.read_tls(&mut cursor)
                .map_err(|e| ExchangeError::NetworkError(format!("TLS read failed: {e}")))?;
            
            // Process any TLS messages
            self.tls_conn.process_new_packets()
                .map_err(|e| ExchangeError::NetworkError(format!("TLS process failed: {e}")))?;

            // Prepare buffer for next iteration
            tcp_buffer = vec![0u8; 4096];
        }

        Ok(response_data)
    }
}

// Implement Read and Write traits for TlsStream to work with tungstenite
impl std::io::Read for TlsStream {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        // This is a blocking operation for WebSocket compatibility
        // In a real implementation, you'd want to use a different approach
        // For now, return an error to indicate this should be handled differently
        Err(std::io::Error::new(
            std::io::ErrorKind::WouldBlock,
            "TlsStream read should use async methods"
        ))
    }
}

impl std::io::Write for TlsStream {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        // This is a blocking operation for WebSocket compatibility
        // In a real implementation, you'd want to use a different approach
        Err(std::io::Error::new(
            std::io::ErrorKind::WouldBlock,
            "TlsStream write should use async methods"
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Default for MonoioHttpsClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTPS client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[monoio::test]
    async fn test_https_client_creation() {
        let client = MonoioHttpsClient::new();
        assert!(client.is_ok());
    }

    #[monoio::test]
    async fn test_url_parsing() {
        let _client = MonoioHttpsClient::new().unwrap();
        // This test would require actual network access
        // In a real implementation, we'd use a mock server
    }
}
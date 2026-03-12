//! Enhanced transport with Engine.IO protocol support

use crate::engineio::{EnginePacket, EnginePacketType, HandshakeResponse};
use crate::error::{Error, Result};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
    MaybeTlsStream, WebSocketStream,
};

/// Transport with Engine.IO protocol support
pub struct EngineTransport {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    session_id: String,
}

impl EngineTransport {
    /// Connect to a Socket.IO server via Engine.IO protocol
    ///
    /// # Arguments
    /// * `url` - Server URL (e.g., "http://localhost:5530")
    pub async fn connect(url: &str) -> Result<Self> {
        // Step 1: Engine.IO handshake via polling
        let session_id = Self::do_handshake(url).await?;

        // Step 2: Upgrade to WebSocket with session ID
        let ws_url = format!(
            "{}/socket.io/?EIO=4&transport=websocket&sid={}",
            url.replace("http://", "ws://").replace("https://", "wss://"),
            session_id
        );

        let request = ws_url
            .into_client_request()
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            ws_stream,
            session_id,
        })
    }

    /// Perform Engine.IO handshake via HTTP polling
    async fn do_handshake(url: &str) -> Result<String> {
        let handshake_url = format!("{}/socket.io/?EIO=4&transport=polling", url);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        let response = client
            .get(&handshake_url)
            .send()
            .await
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::ConnectionFailed(format!(
                "Handshake failed with status: {}",
                response.status()
            )));
        }

        let body = response
            .bytes()
            .await
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        // Parse Engine.IO handshake response
        // Format: "0{"sid":"...","upgrades":["websocket"],...}"
        let body_str = std::str::from_utf8(&body)
            .map_err(|e| Error::ConnectionFailed(format!("Invalid UTF-8 response: {}", e)))?;

        // Skip first character (packet type '0')
        if !body_str.starts_with('0') {
            return Err(Error::ConnectionFailed(
                "Invalid handshake response format".to_string(),
            ));
        }

        let json_str = &body_str[1..];
        let handshake: HandshakeResponse = serde_json::from_str(json_str)
            .map_err(|e| Error::ConnectionFailed(format!("Failed to parse handshake: {}", e)))?;

        Ok(handshake.sid)
    }

    /// Send Socket.IO packet (wrapped in Engine.IO packet)
    pub async fn send(&mut self, data: Bytes) -> Result<()> {
        let engine_packet = EnginePacket::message(data);
        let encoded = engine_packet.encode()?;

        // Send as binary
        let message = Message::Binary(encoded);
        self.ws_stream
            .send(message)
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;
        Ok(())
    }

    /// Receive Socket.IO packet (unwrap Engine.IO packet)
    pub async fn receive(&mut self) -> Result<Option<Bytes>> {
        loop {
            match self.ws_stream.next().await {
                Some(Ok(Message::Binary(data))) => {
                    let engine_packet = EnginePacket::decode(&data)?;

                    match engine_packet.packet_type {
                        EnginePacketType::Message => return Ok(engine_packet.data),
                        EnginePacketType::Ping => {
                            // Auto-respond to ping with pong
                            let pong = EnginePacket::pong();
                            let encoded = pong.encode()?;
                            let message = Message::Binary(encoded);
                            self.ws_stream
                                .send(message)
                                .await
                                .map_err(|e| Error::WebSocket(e.to_string()))?;
                            // Continue loop
                        }
                        EnginePacketType::Close => return Ok(None),
                        _ => return Ok(None),
                    }
                }
                Some(Ok(Message::Close(_))) => return Ok(None),
                Some(Err(e)) => return Err(Error::WebSocket(e.to_string())),
                None => return Ok(None),
                Some(Ok(_)) => {
                    // Ignore other message types, continue loop
                }
            }
        }
    }

    /// Close the connection
    pub async fn close(&mut self) -> Result<()> {
        // Send Engine.IO close packet
        let close_packet = EnginePacket {
            packet_type: EnginePacketType::Close,
            data: None,
        };
        let encoded = close_packet.encode()?;
        let message = Message::Binary(encoded);

        let _ = self.ws_stream.send(message).await;

        self.ws_stream
            .close(None)
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;
        Ok(())
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_engine_handshake() {
        let result = EngineTransport::do_handshake("http://leaf-server:5530").await;
        assert!(result.is_ok());

        let sid = result.unwrap();
        println!("Got session ID: {}", sid);
        assert!(!sid.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_engine_connect() {
        let transport = EngineTransport::connect("http://leaf-server:5530").await;
        assert!(transport.is_ok());

        let transport = transport.unwrap();
        println!("Connected with session ID: {}", transport.session_id());
    }
}

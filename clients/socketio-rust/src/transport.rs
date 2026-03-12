//! WebSocket transport layer for Socket.IO
//!
//! This module handles the low-level WebSocket connection used by Socket.IO.

use crate::error::{Error, Result};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
    MaybeTlsStream, WebSocketStream,
};

/// WebSocket transport for Socket.IO
pub struct Transport {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl Transport {
    /// Connect to a WebSocket server
    ///
    /// # Arguments
    /// * `url` - Server URL (e.g., "http://localhost:3000")
    pub async fn connect(url: &str) -> Result<Self> {
        // Convert HTTP URL to WebSocket URL
        let ws_url = url.replace("http://", "ws://").replace("https://", "wss://");

        let request = ws_url
            .into_client_request()
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        Ok(Self { ws_stream })
    }

    /// Send data to the server
    ///
    /// # Arguments
    /// * `data` - Bytes to send
    pub async fn send(&mut self, data: Bytes) -> Result<()> {
        let message = Message::Binary(data);
        self.ws_stream
            .send(message)
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;
        Ok(())
    }

    /// Receive data from the server
    ///
    /// Returns None if the stream is closed
    pub async fn receive(&mut self) -> Result<Option<Bytes>> {
        match self.ws_stream.next().await {
            Some(Ok(Message::Binary(data))) => Ok(Some(Bytes::from(data))),
            Some(Ok(Message::Text(text))) => Ok(Some(Bytes::from(text))),
            Some(Ok(Message::Close(_))) => Ok(None),
            Some(Err(e)) => Err(Error::WebSocket(e.to_string())),
            None => Ok(None),
            Some(Ok(_)) => Ok(None), // Ignore other message types
        }
    }

    /// Close the connection
    pub async fn close(&mut self) -> Result<()> {
        self.ws_stream
            .close(None)
            .await
            .map_err(|e| Error::WebSocket(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running WebSocket server
    // They should be marked as ignored and run manually or in integration tests

    #[tokio::test]
    #[ignore]
    async fn test_transport_connect() {
        let transport = Transport::connect("ws://echo.websocket.org").await;
        assert!(transport.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_transport_send_receive() {
        let mut transport = Transport::connect("ws://echo.websocket.org")
            .await
            .expect("Connection failed");

        let test_data = Bytes::from("Hello, WebSocket!");

        transport
            .send(test_data.clone())
            .await
            .expect("Send failed");

        let received = transport
            .receive()
            .await
            .expect("Receive failed")
            .expect("No data received");

        assert_eq!(received, test_data);
    }
}

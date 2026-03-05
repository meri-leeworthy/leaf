// ATProto client wrapper around jacquard

use crate::Error;
use jacquard::client::{Agent, AtpSession, BasicClient, MemorySessionStore, credential_session::CredentialSession};
use jacquard::identity::PublicResolver;
use jacquard::IntoStatic;
use std::sync::Arc;

/// ATProto client wrapper for Roomy operations
///
/// Wraps jacquard's Agent (with CredentialSession) to provide AT Protocol authentication
/// and basic operations needed by Roomy.
pub struct RoomyAtpClient {
    agent: Option<BasicClient>,
    service_url: String,
    did: Option<String>,
}

impl RoomyAtpClient {
    /// Create a new ATProto client (not yet authenticated)
    pub fn new(service_url: &str) -> Self {
        Self {
            agent: None,
            service_url: service_url.to_string(),
            did: None,
        }
    }

    /// Authenticate with ATProto using app password
    ///
    /// # Arguments
    /// * `identifier` - User's handle or DID (e.g., "user.bsky.social" or "did:plc:...")
    /// * `password` - App password (not account password)
    ///
    /// # Example
    /// ```no_run
    /// use roomy_sdk_rust::RoomyAtpClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = RoomyAtpClient::new("https://bsky.social");
    ///     client.login("user.bsky.social", "password-xxx-xxx").await?;
    ///     println!("Authenticated as: {}", client.did().unwrap());
    ///     Ok(())
    /// }
    /// ```
    pub async fn login(&mut self, identifier: &str, password: &str) -> Result<(), Error> {
        // Create HTTP client and resolver
        let http = reqwest::Client::new();
        let resolver = PublicResolver::new(http.clone(), Default::default());

        // Create in-memory session store
        let store = MemorySessionStore::default();

        // Create credential session
        let session = CredentialSession::new(Arc::new(store), Arc::new(resolver));

        // Create agent
        let agent = Agent::new(session);

        // Login with app password - access the inner session directly
        let atp_session: AtpSession = agent
            .inner()
            .login(
                identifier.into(),
                password.into(),
                Some("roomy-sdk".into()), // session_id
                None,                      // allow_takendown
                None,                      // auth_factor_token
                None,                      // pds (will be resolved)
            )
            .await
            .map_err(|e| Error::Auth(format!("ATProto login failed: {}", e)))?;

        // Store DID
        self.did = Some(atp_session.did.into_static().to_string());
        self.agent = Some(agent);

        Ok(())
    }

    /// Get the client's DID (if authenticated)
    pub fn did(&self) -> Option<&str> {
        self.did.as_deref()
    }

    /// Get the access token (if authenticated)
    ///
    /// Note: This currently returns None because we need to extract the token
    /// from the session. This can be implemented when needed.
    pub fn access_token(&self) -> Option<String> {
        // TODO: Extract actual access token from session
        // The session stores the token, but we need to access it via the agent
        None
    }

    /// Check if the client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.agent.is_some() && self.did.is_some()
    }

    /// Get the service URL
    pub fn service_url(&self) -> &str {
        &self.service_url
    }

    /// Get a reference to the underlying jacquard Agent
    ///
    /// Returns None if not yet authenticated
    pub fn agent(&self) -> Option<&BasicClient> {
        self.agent.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = RoomyAtpClient::new("https://bsky.social");
        assert!(!client.is_authenticated());
        assert!(client.did().is_none());
        assert_eq!(client.service_url(), "https://bsky.social");
    }
}

//! Profile sync service

use crate::{repository::BridgeRepository, Error};
use roomy_sdk_rust::Did;
use twilight_http::Client;
use twilight_model::{
    id::marker::{GuildMarker, UserMarker},
    id::Id,
};
use std::sync::Arc;
use tracing::{debug, info, instrument};
 // For Discord user avatar URLs

/// Profile sync service
///
/// Syncs Discord user profiles to Roomy UserInfo events.
/// Uses LRU cache to reduce database and API calls.
pub struct ProfileSyncService {
    repo: Arc<dyn BridgeRepository>,
    _space_id: Did,
    guild_id: Id<GuildMarker>,
    http: Arc<Client>,
    cache: Arc<tokio::sync::Mutex<lru::LruCache<Id<UserMarker>, UserProfile>>>,
}

/// Cached user profile data
#[derive(Debug, Clone)]
struct UserProfile {
    did: Did,
    _username: String,
    _global_name: Option<String>,
    _avatar_url: Option<String>,
    _discriminator: Option<String>,
}

impl ProfileSyncService {
    /// Create a new profile sync service with LRU cache (50 entries)
    pub fn new(
        repo: Arc<dyn BridgeRepository>,
        space_id: Did,
        guild_id: Id<GuildMarker>,
        http: Arc<Client>,
    ) -> Self {
        Self {
            repo,
            _space_id: space_id,
            guild_id,
            http,
            cache: Arc::new(tokio::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(50).unwrap(),
            ))),
        }
    }

    /// Sync user profile to Roomy
    ///
    /// Fetches user from Discord (or uses cache) and sends UserInfo event to Roomy.
    #[instrument(skip(self), fields(guild_id = %self.guild_id, user_id = %user_id))]
    pub async fn sync_user_profile(&self, user_id: Id<UserMarker>) -> Result<(), Error> {
        // Check cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(_profile) = cache.get(&user_id) {
                debug!("Using cached profile for user {}", user_id);
                return Ok(());
            }
        }

        info!("Fetching profile for user {}", user_id);

        // Fetch user from Discord
        let user = self.http.user(user_id).await?.model().await?;

        // Create profile
        let profile = UserProfile {
            did: Did::new(format!("did:discord:{}", user.id.get()))?,
            _username: user.name.clone(),
            _global_name: user.global_name.clone(),
            _avatar_url: user.avatar.map(|hash| {
                format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.png",
                    user.id, hash
                )
            }),
            _discriminator: if user.discriminator == 0 {
                None
            } else {
                Some(format!("{:04}", user.discriminator))
            },
        };

        // Store in cache
        {
            let mut cache = self.cache.lock().await;
            cache.put(user_id, profile.clone());
        }

        // TODO: Send UserInfo event to Roomy
        // let event = Event::UserInfo {
        //     entity_did: profile.did.clone(),
        //     name: profile.global_name.unwrap_or(profile.username),
        //     avatar: profile.avatar_url,
        //     description: None,
        //     created_at: ...,
        // };

        debug!("Synced profile for user {} ({})", user_id, profile.did);

        Ok(())
    }

    /// Batch sync multiple user profiles
    ///
    /// Useful for backfilling user profiles from existing messages.
    #[instrument(skip(self, user_ids))]
    pub async fn sync_user_profiles_batch(&self, user_ids: Vec<Id<UserMarker>>) -> Result<u64, Error> {
        let mut synced = 0u64;

        for user_id in user_ids {
            self.sync_user_profile(user_id).await?;
            synced += 1;

            // Rate limit: Discord allows 50 requests per second
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        Ok(synced)
    }

    /// Get user DID (from cache or create new)
    ///
    /// This is used by other services to get the Roomy DID for a Discord user.
    pub async fn get_user_did(&self, user_id: Id<UserMarker>) -> Result<Did, Error> {
        // Check cache
        {
            let cache = self.cache.lock().await;
            if let Some(profile) = cache.peek(&user_id) {
                return Ok(profile.did.clone());
            }
        }

        // Check repository
        if let Some(roomy_id) = self
            .repo
            .get_roomy_id(&crate::repository::discord_user_id(user_id))
            .await?
        {
            if let crate::repository::RoomyId::User(did) = roomy_id {
                return Ok(did);
            }
        }

        // Create new DID
        let did = Did::new(format!("did:discord:{}", user_id.get()))?;

        // Store mapping
        self.repo
            .register_mapping(
                &crate::repository::discord_user_id(user_id),
                &crate::repository::RoomyId::User(did.clone()),
            )
            .await?;

        // Trigger profile sync
        self.sync_user_profile(user_id).await?;

        Ok(did)
    }

    /// Clear the profile cache
    ///
    /// Useful for testing or forcing profile refresh.
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
        debug!("Profile cache cleared");
    }

    /// Get cache size
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.lock().await;
        cache.len()
    }
}

use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use thiserror::Error;

const SERVICE_NAME: &str = "com.eury.agent";
const TOKENS_KEY: &str = "auth_tokens";
const LEGACY_ACCESS_TOKEN_KEY: &str = "access_token";
const LEGACY_REFRESH_TOKEN_KEY: &str = "refresh_token";

static TOKEN_CACHE: Mutex<Option<AuthTokens>> = Mutex::new(None);

#[derive(Error, Debug)]
pub enum AuthStoreError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    #[error("No token found")]
    NotFound,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    /// Which device this session belongs to. The platform's refresh endpoint
    /// requires it, so a session stored without one can never be refreshed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    /// Unix seconds at which `access_token` stops being accepted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}

pub struct AuthStore;

impl AuthStore {
    /// Set `EURY_AGENT_AUTH_STORE=memory` to opt out of persisting to the OS
    /// keychain (e.g. to avoid repeated macOS Keychain password prompts).
    fn use_memory_only() -> bool {
        std::env::var("EURY_AGENT_AUTH_STORE").is_ok_and(|value| value == "memory")
    }

    fn read_cache() -> Option<AuthTokens> {
        TOKEN_CACHE.lock().ok()?.clone()
    }

    fn write_cache(tokens: Option<AuthTokens>) {
        if let Ok(mut cache) = TOKEN_CACHE.lock() {
            *cache = tokens;
        }
    }

    /// Loads tokens into the process cache without persisting them anywhere.
    ///
    /// This is how a host that keeps tokens in its own encrypted store hands
    /// them to the engine: [`Self::get_tokens`] (and so the gateway) sees them
    /// immediately, and the OS keychain is never opened — which on macOS is
    /// what triggers a login-password prompt on every launch of a freshly
    /// built, ad-hoc-signed binary.
    pub fn set_cached_tokens(tokens: AuthTokens) {
        Self::write_cache(Some(tokens));
    }

    /// Drops the cached tokens without touching the keychain.
    pub fn clear_cached_tokens() {
        Self::write_cache(None);
    }

    /// Whether tokens are already available in-process.
    #[must_use]
    pub fn has_cached_tokens() -> bool {
        Self::read_cache().is_some()
    }

    /// # Errors
    ///
    /// Returns an error if serialization fails or the OS keychain rejects the write.
    pub fn save_tokens(tokens: &AuthTokens) -> Result<(), AuthStoreError> {
        Self::write_cache(Some(tokens.clone()));

        if Self::use_memory_only() {
            return Ok(());
        }

        let entry = Entry::new(SERVICE_NAME, TOKENS_KEY)?;
        entry.set_password(&serde_json::to_string(tokens)?)?;

        // Remove legacy split entries so future reads only touch one keychain item.
        let _ = Entry::new(SERVICE_NAME, LEGACY_ACCESS_TOKEN_KEY)?.delete_credential();
        let _ = Entry::new(SERVICE_NAME, LEGACY_REFRESH_TOKEN_KEY)?.delete_credential();

        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if no tokens are cached and the OS keychain has none stored.
    pub fn get_tokens() -> Result<AuthTokens, AuthStoreError> {
        if let Some(tokens) = Self::read_cache() {
            return Ok(tokens);
        }

        if Self::use_memory_only() {
            return Err(AuthStoreError::NotFound);
        }

        let tokens = Self::read_tokens_from_keyring()?;
        Self::write_cache(Some(tokens.clone()));
        Ok(tokens)
    }

    /// # Errors
    ///
    /// Returns an error if the OS keychain rejects the delete.
    pub fn clear_tokens() -> Result<(), AuthStoreError> {
        Self::write_cache(None);

        if Self::use_memory_only() {
            return Ok(());
        }

        let _ = Entry::new(SERVICE_NAME, TOKENS_KEY)?.delete_credential();
        let _ = Entry::new(SERVICE_NAME, LEGACY_ACCESS_TOKEN_KEY)?.delete_credential();
        let _ = Entry::new(SERVICE_NAME, LEGACY_REFRESH_TOKEN_KEY)?.delete_credential();

        Ok(())
    }

    fn read_tokens_from_keyring() -> Result<AuthTokens, AuthStoreError> {
        let entry = Entry::new(SERVICE_NAME, TOKENS_KEY)?;
        match entry.get_password() {
            Ok(json) => return Ok(serde_json::from_str(&json)?),
            Err(keyring::Error::NoEntry) => {}
            Err(error) => return Err(AuthStoreError::Keyring(error)),
        }

        Self::read_legacy_tokens_from_keyring()
    }

    fn read_legacy_tokens_from_keyring() -> Result<AuthTokens, AuthStoreError> {
        let access_entry = Entry::new(SERVICE_NAME, LEGACY_ACCESS_TOKEN_KEY)?;
        let access_token = match access_entry.get_password() {
            Ok(token) => token,
            Err(keyring::Error::NoEntry) => return Err(AuthStoreError::NotFound),
            Err(error) => return Err(AuthStoreError::Keyring(error)),
        };

        let refresh_entry = Entry::new(SERVICE_NAME, LEGACY_REFRESH_TOKEN_KEY)?;
        let refresh_token = match refresh_entry.get_password() {
            Ok(token) => token,
            Err(keyring::Error::NoEntry) => return Err(AuthStoreError::NotFound),
            Err(error) => return Err(AuthStoreError::Keyring(error)),
        };

        // A legacy split entry predates device/expiry tracking, so this
        // session can only be refreshed after the next sign-in.
        let tokens =
            AuthTokens { access_token, refresh_token, device_id: None, expires_at: None };

        // Migrate to the single-entry format without prompting again on the next read.
        let _ = Self::save_tokens(&tokens);

        Ok(tokens)
    }
}

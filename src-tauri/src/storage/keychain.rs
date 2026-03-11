use anyhow::{Context, Result};
use keyring::Entry;
use rand::RngCore;
use zeroize::Zeroizing;

const SERVICE: &str = "com.meridian.app";

pub struct Keychain;

impl Keychain {
    pub fn new() -> Result<Self> { Ok(Self) }

    // ── DB encryption key ────────────────────────────────────────────────────

    /// Fetch existing DB key or generate a new 256-bit key and store it.
    pub fn get_or_create_db_key(&self) -> Result<[u8; 32]> {
        let entry = Entry::new(SERVICE, "db.master_key")
            .context("Keychain entry creation failed")?;

        match entry.get_password() {
            Ok(hex) => {
                let bytes = hex::decode(&hex).context("Invalid DB key in keychain")?;
                let mut key = [0u8; 32];
                key.copy_from_slice(&bytes[..32]);
                Ok(key)
            }
            Err(_) => {
                // Generate fresh key
                let mut key = Zeroizing::new([0u8; 32]);
                rand::thread_rng().fill_bytes(key.as_mut());
                entry
                    .set_password(&hex::encode(*key))
                    .context("Could not store DB key in OS keychain")?;
                Ok(*key)
            }
        }
    }

    // ── OAuth tokens ─────────────────────────────────────────────────────────

    pub fn store_token(&self, integration_id: &str, token_json: &str) -> Result<()> {
        let account = format!("token.{integration_id}");
        let entry = Entry::new(SERVICE, &account)
            .context("Keychain entry creation failed")?;
        entry.set_password(token_json).context("Cannot store token")
    }

    pub fn get_token(&self, integration_id: &str) -> Result<String> {
        let account = format!("token.{integration_id}");
        let entry = Entry::new(SERVICE, &account)
            .context("Keychain entry creation failed")?;
        entry.get_password().context("Token not found in keychain")
    }

    pub fn delete_token(&self, integration_id: &str) -> Result<()> {
        let account = format!("token.{integration_id}");
        let entry = Entry::new(SERVICE, &account)
            .context("Keychain entry creation failed")?;
        entry.delete_password().context("Cannot delete token")
    }

    pub fn token_exists(&self, integration_id: &str) -> bool {
        let account = format!("token.{integration_id}");
        Entry::new(SERVICE, &account)
            .and_then(|e| e.get_password())
            .is_ok()
    }

    // ── Account metadata (non-secret) ────────────────────────────────────────

    pub fn store_account(&self, integration_id: &str, email: &str) -> Result<()> {
        let account = format!("account.{integration_id}");
        let entry = Entry::new(SERVICE, &account)
            .context("Keychain entry creation failed")?;
        entry.set_password(email).context("Cannot store account")
    }

    pub fn get_account(&self, integration_id: &str) -> Result<String> {
        let account = format!("account.{integration_id}");
        let entry = Entry::new(SERVICE, &account)
            .context("Keychain entry creation failed")?;
        entry.get_password().context("Account not found")
    }
}

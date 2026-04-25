use anyhow::Result;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcManConfig {
    pub hub_addr: String,
    pub public_addr: String,
    pub installation_id: Uuid,
    pub port_range: (u16, u16),
    pub library_dir: String,
    pub max_total_instances: usize,
    /// HMAC-SHA256 key (64 hex chars = 32 bytes) used to sign per-room JWT
    /// tickets. The Hub uses this to issue tickets; the dedicated server
    /// receives it via stdin and validates incoming connection tickets.
    /// Auto-generated on first run. Must be consistent for the lifetime of
    /// this procman instance; rotate only when cycling servers.
    pub ticket_hmac_secret: String,
}

/// Generates a cryptographically random 256-bit HMAC secret, hex-encoded.
pub fn generate_hmac_secret() -> String {
    let mut rng = rand::rng();
    let bytes: [u8; 32] = rng.random();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub async fn load_config(path: impl AsRef<Path>) -> Result<ProcManConfig> {
    if !path.as_ref().exists() {
        let default_config = ProcManConfig {
            hub_addr: "localhost:11000".to_string(),
            public_addr: "127.0.0.1".to_string(),
            installation_id: Uuid::new_v4(),
            port_range: (12000, 12100),
            library_dir: "./library".to_string(),
            max_total_instances: 8,
            ticket_hmac_secret: generate_hmac_secret(),
        };
        save_config(path, &default_config).await?;
        return Ok(default_config);
    }

    let content = tokio::fs::read_to_string(path).await?;
    let config = ron::from_str(&content)?;
    Ok(config)
}

pub async fn save_config(path: impl AsRef<Path>, config: &ProcManConfig) -> Result<()> {
    let content = ron::ser::to_string_pretty(config, ron::ser::PrettyConfig::default())?;
    tokio::fs::write(path, content).await?;
    Ok(())
}

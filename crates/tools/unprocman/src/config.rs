use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcManConfig {
    pub hub_addr: String,
    pub public_addr: String,
    pub installation_id: Uuid,
    pub port_range: (u16, u16),
    pub idle_pool_size: usize,
    pub game_binary_path: String,
}

pub async fn load_config(path: impl AsRef<Path>) -> Result<ProcManConfig> {
    if !path.as_ref().exists() {
        let default_config = ProcManConfig {
            hub_addr: "localhost:11000".to_string(),
            public_addr: "127.0.0.1".to_string(),
            installation_id: Uuid::new_v4(),
            port_range: (12000, 12100),
            idle_pool_size: 1,
            game_binary_path: "./unhaunter_dedicated".to_string(),
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

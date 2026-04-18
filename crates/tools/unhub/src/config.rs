use crate::state::HubConfig;
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

pub async fn load_config(path: impl AsRef<Path>) -> Result<HubConfig> {
    if !path.as_ref().exists() {
        let default_config = HubConfig {
            version: 1,
            api_bind: "127.0.0.1:3000".to_string(),
            procman_bind: "127.0.0.1:11000".to_string(),
            official_server_keys: Default::default(),
            banned_uuids: HashSet::new(),
            allowed_procman_uuids: HashSet::new(),
            max_rooms_per_ip: 2,
            trust_proxy_headers: false,
            pow_difficulty: 20,
        };
        save_config(path, &default_config).await?;
        return Ok(default_config);
    }

    let content = tokio::fs::read_to_string(path).await?;
    let config = ron::from_str(&content)?;
    Ok(config)
}

pub async fn save_config(path: impl AsRef<Path>, config: &HubConfig) -> Result<()> {
    let content = ron::ser::to_string_pretty(config, ron::ser::PrettyConfig::default())?;
    let tmp_path = path.as_ref().with_extension("tmp");
    tokio::fs::write(&tmp_path, content).await?;
    tokio::fs::rename(tmp_path, path).await?;
    Ok(())
}

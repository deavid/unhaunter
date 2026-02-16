use crate::state::HubConfig;
use std::collections::HashSet;
use std::path::Path;
use anyhow::Result;

pub async fn load_config(path: impl AsRef<Path>) -> Result<HubConfig> {
    if !path.as_ref().exists() {
        let default_config = HubConfig {
            version: 1,
            official_server_keys: Default::default(),
            banned_uuids: HashSet::new(),
            allowed_procman_uuids: HashSet::new(),
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

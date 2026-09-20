use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use log::{info, warn};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FanConfig {
    pub fan_mode: String,
    pub custom_curve: String,
    pub thermal_protection_enabled: bool,
}

impl Default for FanConfig {
    fn default() -> Self {
        Self {
            fan_mode: "auto".to_string(),
            custom_curve: "[]".to_string(),
            thermal_protection_enabled: true,
        }
    }
}

pub struct ConfigManager {
    path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let path = PathBuf::from("/var/lib/omen-space-daemon/fan_config.json");
        Self { path }
    }

    pub async fn load(&self) -> FanConfig {
        match fs::read_to_string(&self.path).await {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(config) => {
                        info!("Loaded fan config from {:?}", self.path);
                        config
                    }
                    Err(e) => {
                        warn!("Failed to parse config file {:?}: {}. Using defaults.", self.path, e);
                        FanConfig::default()
                    }
                }
            }
            Err(_) => {
                info!("No existing config found at {:?}. Using defaults.", self.path);
                FanConfig::default()
            }
        }
    }

    pub async fn save(&self, config: &FanConfig) {
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent).await;
        }

        match serde_json::to_string_pretty(config) {
            Ok(json) => {
                if let Err(e) = fs::write(&self.path, json).await {
                    warn!("Failed to save fan config to {:?}: {}", self.path, e);
                } else {
                    info!("Saved fan config to {:?}", self.path);
                }
            }
            Err(e) => {
                warn!("Failed to serialize fan config: {}", e);
            }
        }
    }
}

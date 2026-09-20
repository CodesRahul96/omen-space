use log::{info, error};
use std::collections::HashMap;
use std::sync::Arc;
use std::fs;
use std::path::Path;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use crate::notifier::DesktopNotifier;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppProfile {
    pub process_name: String,
    pub power_profile: String,
    pub fan_mode: String,
}

#[derive(Clone)]
pub struct GameAutomationService {
    game_profiles: Arc<Mutex<HashMap<String, AppProfile>>>, // process_name -> AppProfile
    active_game: Arc<Mutex<Option<String>>>,
}

impl GameAutomationService {
    pub fn new() -> Self {
        let default_games = Self::load_profiles();
        Self {
            game_profiles: Arc::new(Mutex::new(default_games)),
            active_game: Arc::new(Mutex::new(None)),
        }
    }

    fn load_profiles() -> HashMap<String, AppProfile> {
        let path = Path::new("/etc/omenspace/app_profiles.json");
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(profiles) = serde_json::from_str::<Vec<AppProfile>>(&content) {
                let mut map = HashMap::new();
                for p in profiles {
                    map.insert(p.process_name.clone(), p);
                }
                return map;
            }
        }
        HashMap::new()
    }

    fn save_profiles(map: &HashMap<String, AppProfile>) {
        let path = Path::new("/etc/omenspace/app_profiles.json");
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let profiles: Vec<AppProfile> = map.values().cloned().collect();
        if let Ok(json) = serde_json::to_string_pretty(&profiles) {
            if let Err(e) = fs::write(path, json) {
                error!("Failed to save app profiles: {}", e);
            }
        }
    }

    pub fn start_monitor(&self) {
        let profiles = self.game_profiles.clone();
        let active = self.active_game.clone();

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(10)).await;
                let running_procs = crate::sysmon::get_running_process_names();

                let game_map = profiles.lock().await.clone();
                let mut found_game: Option<AppProfile> = None;

                for (proc_name, profile) in game_map {
                    if running_procs.contains(&proc_name.to_lowercase()) {
                        found_game = Some(profile);
                        break;
                    }
                }

                let mut active_lock = active.lock().await;
                match (active_lock.clone(), found_game) {
                    (None, Some(profile)) => {
                        info!("Detected game start: {}. Switching to '{}' profile...", profile.process_name, profile.power_profile);
                        *active_lock = Some(profile.process_name.clone());
                        DesktopNotifier::send_notification(
                            "OMENSpace App Profile Activated",
                            &format!("App '{}' launched. Switched performance profile to '{}'.", profile.process_name, profile.power_profile),
                            1,
                        ).await;
                        // Apply performance profile via platform
                        let _ = crate::platform::set_thermal_policy_by_name(&profile.power_profile);
                    }
                    (Some(current), None) => {
                        info!("Game '{}' closed. Restoring default profile...", current);
                        *active_lock = None;
                        DesktopNotifier::send_notification(
                            "OMENSpace Game Profile Deactivated",
                            &format!("Game '{}' closed. Restored default performance profile.", current),
                            0,
                        ).await;
                        let _ = crate::platform::set_thermal_policy_by_name("Balanced");
                    }
                    _ => {}
                }
            }
        });
    }
}

#[zbus::interface(name = "org.hp.omen.AppProfiles")]
impl GameAutomationService {
    pub async fn get_profiles(&self) -> String {
        let map = self.game_profiles.lock().await;
        let profiles: Vec<AppProfile> = map.values().cloned().collect();
        serde_json::to_string(&profiles).unwrap_or_default()
    }

    pub async fn add_profile(&self, process_name: String, power_profile: String, fan_mode: String) -> String {
        let mut map = self.game_profiles.lock().await;
        let proc_lower = process_name.to_lowercase();
        map.insert(proc_lower.clone(), AppProfile {
            process_name: proc_lower,
            power_profile,
            fan_mode,
        });
        Self::save_profiles(&map);
        "OK".to_string()
    }

    pub async fn remove_profile(&self, process_name: String) -> String {
        let mut map = self.game_profiles.lock().await;
        map.remove(&process_name.to_lowercase());
        Self::save_profiles(&map);
        "OK".to_string()
    }
}

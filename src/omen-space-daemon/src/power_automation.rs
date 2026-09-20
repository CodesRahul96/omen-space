use log::info;
use zbus::fdo::PropertiesProxy;
use futures::StreamExt;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use crate::notifier::DesktopNotifier;
use std::sync::OnceLock;
use std::path::PathBuf;

static AC_ONLINE_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

#[derive(Clone)]
pub struct PowerAutomationService {
    last_ac_state: Arc<Mutex<Option<bool>>>,
}

impl PowerAutomationService {
    pub fn new() -> Self {
        Self {
            last_ac_state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_monitor(&self) {
        let last_state = self.last_ac_state.clone();

        tokio::spawn(async move {
            if let Ok(conn) = zbus::Connection::system().await {
                if let Ok(proxy) = PropertiesProxy::builder(&conn)
                    .destination("net.hadess.PowerProfiles").unwrap()
                    .path("/net/hadess/PowerProfiles").unwrap()
                    .build().await {
                    
                    if let Ok(mut stream) = proxy.receive_properties_changed().await {
                        while let Some(signal) = stream.next().await {
                            if let Ok(args) = signal.args() {
                                if args.interface_name() == "net.hadess.PowerProfiles" {
                                    if let Some(val) = args.changed_properties().get("ActiveProfile") {
                                        if let Ok(profile_str) = <&str>::try_from(val) {
                                            info!("PPD Profile changed to: {}", profile_str);
                                            let mode = match profile_str {
                                                "performance" => "Performance",
                                                "power-saver" => "Quiet",
                                                _ => "Default",
                                            };
                                            let _ = crate::platform::set_thermal_policy_by_name(mode);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(4)).await;
                let current_ac = is_ac_power_connected();

                let mut state_lock = last_state.lock().await;
                if let Some(prev) = *state_lock {
                    if prev != current_ac {
                        *state_lock = Some(current_ac);
                        if current_ac {
                            info!("AC Power connected. Applying AC Performance Profile...");
                            DesktopNotifier::send_notification(
                                "OMENSpace Power Automation",
                                "AC Power connected. Switched to Performance mode.",
                                0,
                            ).await;
                            let _ = crate::platform::set_thermal_policy_by_name("Performance");
                        } else {
                            info!("Battery Power connected. Applying Battery Saver Profile...");
                            DesktopNotifier::send_notification(
                                "OMENSpace Power Automation",
                                "Running on Battery. Switched to Quiet/Saver mode.",
                                1,
                            ).await;
                            let _ = crate::platform::set_thermal_policy_by_name("Quiet");
                        }
                    }
                } else {
                    *state_lock = Some(current_ac);
                }
            }
        });
    }
}

fn is_ac_power_connected() -> bool {
    let online_path = AC_ONLINE_PATH.get_or_init(|| {
        if let Ok(entries) = fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("AC") || name.starts_with("ADP") || name.starts_with("ac") {
                        return Some(path.join("online"));
                    }
                }
            }
        }
        None
    });

    if let Some(p) = online_path {
        if let Ok(val) = fs::read_to_string(p) {
            return val.trim() == "1";
        }
    }
    true // default to AC if undetectable
}

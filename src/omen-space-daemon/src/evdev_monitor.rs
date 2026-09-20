use std::sync::Arc;
use tokio::sync::Mutex;
use evdev::{Device, Key};
use futures::StreamExt;
use std::sync::atomic::{AtomicBool, Ordering};
use log::info;

#[derive(Clone, Debug)]
pub struct KeyEventInfo {
    #[allow(dead_code)]
    pub key_code: u16,
    pub x: f64,
    pub y: f64,
    pub timestamp: std::time::Instant,
}

pub struct EvdevMonitor {
    pub recent_keys: Arc<Mutex<Vec<KeyEventInfo>>>,
    pub active: Arc<AtomicBool>,
}

impl EvdevMonitor {
    pub fn new() -> Self {
        let recent_keys = Arc::new(Mutex::new(Vec::new()));
        let active = Arc::new(AtomicBool::new(false));
        
        let keys_clone = recent_keys.clone();
        let active_clone = active.clone();
        tokio::spawn(async move {
            Self::monitor_loop(keys_clone, active_clone).await;
        });

        Self { recent_keys, active }
    }

    pub fn set_active(&self, enabled: bool) {
        let prev = self.active.swap(enabled, Ordering::Relaxed);
        if prev && !enabled {
            let keys = self.recent_keys.clone();
            tokio::spawn(async move {
                let mut lock = keys.lock().await;
                lock.clear();
            });
        }
    }

    async fn monitor_loop(keys: Arc<Mutex<Vec<KeyEventInfo>>>, active: Arc<AtomicBool>) {
        loop {
            let mut streams = Vec::new();

            if let Ok(mut dir) = tokio::fs::read_dir("/dev/input").await {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let path = entry.path();
                    if path.to_string_lossy().contains("event") {
                        if let Ok(dev) = Device::open(&path) {
                            if dev.supported_keys().is_some_and(|k| k.contains(Key::KEY_A)) {
                                let is_mouse = dev.supported_relative_axes().is_some_and(|a| a.contains(evdev::RelativeAxisType::REL_X) || a.contains(evdev::RelativeAxisType::REL_Y));
                                if !is_mouse {
                                    if let Ok(stream) = dev.into_event_stream() {
                                        info!("EvdevMonitor: listening to {:?}", path);
                                        streams.push(stream);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if streams.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }

            let mut select_all = futures::stream::select_all(streams);

            while let Some(Ok(event)) = select_all.next().await {
                if !active.load(Ordering::Relaxed) {
                    continue;
                }
                if let evdev::InputEventKind::Key(key) = event.kind() {
                    if event.value() == 1 {
                        let (x, y) = Self::map_keycode(key.code());
                        let mut lock = keys.lock().await;
                        lock.push(KeyEventInfo {
                            key_code: key.code(),
                            x,
                            y,
                            timestamp: std::time::Instant::now(),
                        });
                        if lock.len() > 30 {
                            lock.remove(0);
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    fn map_keycode(code: u16) -> (f64, f64) {
        // Approximate mapping mapping to 0..14 (x) and 0..5 (y)
        let x = code as f64 % 15.0;
        if code == Key::KEY_ESC.code() || (code >= Key::KEY_F1.code() && code <= Key::KEY_F12.code()) {
            (x, 0.0)
        } else if code == Key::KEY_GRAVE.code() || (code >= Key::KEY_1.code() && code <= Key::KEY_EQUAL.code()) || code == Key::KEY_BACKSPACE.code() {
            (x, 1.0)
        } else if code == Key::KEY_TAB.code() || (code >= Key::KEY_Q.code() && code <= Key::KEY_RIGHTBRACE.code()) || code == Key::KEY_BACKSLASH.code() {
            (x, 2.0)
        } else if code == Key::KEY_CAPSLOCK.code() || (code >= Key::KEY_A.code() && code <= Key::KEY_APOSTROPHE.code()) || code == Key::KEY_ENTER.code() {
            (x, 3.0)
        } else if code == Key::KEY_LEFTSHIFT.code() || (code >= Key::KEY_Z.code() && code <= Key::KEY_SLASH.code()) || code == Key::KEY_RIGHTSHIFT.code() {
            (x, 4.0)
        } else if code == Key::KEY_LEFTCTRL.code() || code == Key::KEY_LEFTMETA.code() || code == Key::KEY_LEFTALT.code() || code == Key::KEY_SPACE.code() {
            (7.0, 5.0)
        } else if code == Key::KEY_UP.code() || code == Key::KEY_DOWN.code() || code == Key::KEY_LEFT.code() || code == Key::KEY_RIGHT.code() {
            (12.0, 5.0)
        } else {
            (7.0, 2.0)
        }
    }
}

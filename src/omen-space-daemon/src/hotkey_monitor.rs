use evdev::{Device, Key};
use futures::StreamExt;
use log::info;
use zbus::Connection;

pub struct HotkeyMonitor;

impl HotkeyMonitor {
    pub fn start(connection: Connection) {
        tokio::spawn(async move {
            Self::monitor_loop(connection).await;
        });
    }

    async fn monitor_loop(connection: Connection) {
        loop {
            let mut streams = Vec::new();
            if let Ok(mut dir) = tokio::fs::read_dir("/dev/input").await {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let path = entry.path();
                    if path.to_string_lossy().contains("event") {
                        if let Ok(dev) = Device::open(&path) {
                            let is_keyboard = dev.supported_keys().is_some_and(|k| {
                                k.contains(Key::KEY_A) || k.contains(Key::KEY_F2) || k.contains(Key::KEY_PROG1) || k.contains(Key::KEY_CALC)
                            });
                            let is_mouse = dev.supported_relative_axes().is_some_and(|a| {
                                a.contains(evdev::RelativeAxisType::REL_X) || a.contains(evdev::RelativeAxisType::REL_Y)
                            });
                            if is_keyboard && !is_mouse {
                                if let Ok(stream) = dev.into_event_stream() {
                                    info!("HotkeyMonitor: listening to {:?}", path);
                                    streams.push(stream);
                                }
                            }
                        }
                    }
                }
            }

            if streams.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                continue;
            }

            let mut select_all = futures::stream::select_all(streams);
            let mut left_shift = false;
            let mut right_shift = false;

            while let Some(Ok(event)) = select_all.next().await {
                if let evdev::InputEventKind::Key(key) = event.kind() {
                    let key_code = key.code();

                    // Track Shift key state
                    if key_code == Key::KEY_LEFTSHIFT.code() {
                        left_shift = event.value() != 0;
                    } else if key_code == Key::KEY_RIGHTSHIFT.code() {
                        right_shift = event.value() != 0;
                    }
                    let shift_held = left_shift || right_shift;

                    if event.value() == 1 { // Key press
                        let key_name = if (key_code == Key::KEY_F2.code() || key_code == 60) && shift_held {
                            Some("overlay")
                        } else {
                            // 148 = KEY_PROG1 (Omen Key mapped by hwdb)
                            // 149 = KEY_PROG2 (P1/P2/Macro)
                            // 140 = KEY_CALC (Calculator)
                            match key_code {
                                148 => Some("omen"),
                                149 => Some("prog2"),
                                140 => Some("calc"),
                                256 => Some("prog3"),
                                _ => None,
                            }
                        };

                        if let Some(name) = key_name {
                            info!("HotkeyMonitor: Detected Hotkey / Macro Key: {}", name);
                            let _ = connection.emit_signal(
                                None::<&str>,
                                "/org/hp/omen/Platform",
                                "org.hp.omen.Platform",
                                "MacroKeyPressed",
                                &(name),
                            ).await;
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}

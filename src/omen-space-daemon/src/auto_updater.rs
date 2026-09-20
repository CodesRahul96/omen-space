use serde::{Deserialize, Serialize};
use log::{info, warn};
use crate::notifier::DesktopNotifier;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REPO_OWNER_NAME: &str = "yunusemreyl/omen-space";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_notes: String,
    pub release_url: String,
    pub download_url: String,
    pub status_message: String,
}

pub struct AutoUpdateService;

impl AutoUpdateService {
    pub async fn check_for_updates() -> AppUpdateInfo {
        info!("Omen Space: Checking GitHub Releases for application updates (Current: v{})...", CURRENT_VERSION);

        let (latest_tag, notes, release_url, download_url) = fetch_latest_github_release().await;

        let update_available = is_newer_semver(CURRENT_VERSION, &latest_tag);

        let status_message = if update_available {
            format!("New Omen Space release '{}' available! (Current: v{})", latest_tag, CURRENT_VERSION)
        } else {
            format!("Omen Space is up to date (v{})", CURRENT_VERSION)
        };

        info!("Omen Space Update Check: {}", status_message);

        if update_available {
            DesktopNotifier::send_notification(
                "Omen Space Update Available",
                &format!("A new release '{}' is available for Omen Space! Current version: v{}.", latest_tag, CURRENT_VERSION),
                0,
            ).await;
        }

        AppUpdateInfo {
            current_version: CURRENT_VERSION.to_string(),
            latest_version: latest_tag,
            update_available,
            release_notes: notes,
            release_url,
            download_url,
            status_message,
        }
    }

    pub async fn apply_update() -> String {
        info!("Starting Omen Space application auto-update...");
        let info = Self::check_for_updates().await;

        if !info.update_available {
            return serde_json::json!({
                "success": false,
                "message": format!("No update needed. Already running latest version (v{})", CURRENT_VERSION)
            }).to_string();
        }

        DesktopNotifier::send_notification(
            "Omen Space Updating",
            &format!("Downloading and installing Omen Space {}...", info.latest_version),
            0,
        ).await;

        let update_dir = "/var/lib/omen-space/updates";
        let _ = tokio::fs::create_dir_all(update_dir).await;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = tokio::fs::set_permissions(update_dir, std::fs::Permissions::from_mode(0o700)).await;
        }

        let download_target = format!("{}/omen-space-update.tar.gz", update_dir);
        let extract_dir = format!("{}/extracted", update_dir);

        // Security Check: Validate download URL scheme & host
        if !info.download_url.starts_with("https://github.com/yunusemreyl/omen-space/") {
            warn!("Blocked unsafe download URL: {}", info.download_url);
            let _ = tokio::fs::remove_dir_all(update_dir).await;
            return serde_json::json!({
                "success": false,
                "message": "Blocked update: Invalid or untrusted download URL host."
            }).to_string();
        }

        let _ = tokio::fs::remove_dir_all(&extract_dir).await;
        let _ = tokio::fs::create_dir_all(&extract_dir).await;

        // Safe direct execution of curl with TLS 1.2+ enforcement (No shell invocation)
        let dl_status = tokio::process::Command::new("curl")
            .args(["--proto", "=https", "--tlsv1.2", "-sSL", "--max-redirs", "5", "-o", &download_target, &info.download_url])
            .output()
            .await;

        let target_metadata = tokio::fs::metadata(&download_target).await;
        let file_valid = target_metadata.map(|m| m.len() > 1024).unwrap_or(false);

        if dl_status.is_err() || !file_valid {
            warn!("Failed or invalid update download asset from {}", info.download_url);
            let _ = tokio::fs::remove_dir_all(update_dir).await;
            return serde_json::json!({
                "success": false,
                "message": format!("Failed to securely download release asset from {}", info.download_url)
            }).to_string();
        }

        // Extract package safely
        let extract_cmd = tokio::process::Command::new("tar")
            .args(["-xzf", &download_target, "-C", &extract_dir])
            .output()
            .await;

        if extract_cmd.is_err() || !extract_cmd.as_ref().map(|o| o.status.success()).unwrap_or(false) {
            warn!("Failed to extract update package {}", download_target);
            let _ = tokio::fs::remove_dir_all(update_dir).await;
            return serde_json::json!({
                "success": false,
                "message": "Failed to unpack release archive."
            }).to_string();
        }

        // Check for extracted binary & verify ELF magic bytes
        let new_binary = format!("{}/omen-space-daemon", extract_dir);
        let installed_path = "/usr/libexec/omen-space/omen-space-daemon";

        if tokio::fs::try_exists(&new_binary).await.unwrap_or(false) {
            // Verify ELF magic bytes [0x7F, b'E', b'L', b'F']
            if let Ok(bytes) = tokio::fs::read(&new_binary).await {
                if bytes.len() > 4 && &bytes[0..4] == b"\x7FELF" {
                    let copy_cmd = tokio::process::Command::new("cp")
                        .args(["-f", &new_binary, installed_path])
                        .output()
                        .await;

                    if let Ok(out) = copy_cmd {
                        if out.status.success() {
                            let _ = tokio::process::Command::new("chmod").args(["+x", installed_path]).output().await;
                            info!("Successfully updated Omen Space binary to {}", info.latest_version);
                            DesktopNotifier::send_notification(
                                "Omen Space Updated!",
                                &format!("Omen Space has been successfully updated to {}!", info.latest_version),
                                0,
                            ).await;

                            let _ = tokio::fs::remove_dir_all(update_dir).await;

                            return serde_json::json!({
                                "success": true,
                                "version": info.latest_version,
                                "message": "Update installed successfully. Executable updated."
                            }).to_string();
                        }
                    }
                } else {
                    warn!("Downloaded binary failed ELF magic header validation.");
                }
            }
        }

        // Fallback: Notify user of downloaded update package
        let mut target_dir = extract_dir.clone();
        if let Ok(mut entries) = tokio::fs::read_dir(&extract_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.file_type().await.map(|f| f.is_dir()).unwrap_or(false)
                    && entry.file_name().to_string_lossy().starts_with("omen-space") {
                        target_dir = entry.path().to_string_lossy().to_string();
                        break;
                    }
            }
        }

        let extract_dir_clone = target_dir.clone();
        tokio::task::spawn_blocking(move || {
            DesktopNotifier::open_in_user_session(&extract_dir_clone);
        });

        serde_json::json!({
            "success": true,
            "version": info.latest_version,
            "message": format!("Update downloaded to {}. Please run 'sudo ./setup.sh update' inside that folder.", target_dir)
        }).to_string()
    }
}

async fn fetch_latest_github_release() -> (String, String, String, String) {
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", REPO_OWNER_NAME);
    let output = tokio::process::Command::new("curl")
        .args(["--proto", "=https", "--tlsv1.2", "-s", "-H", "User-Agent: OmenSpace-Daemon", &api_url])
        .output()
        .await;

    if let Ok(out) = output {
        let json_str = String::from_utf8_lossy(&out.stdout);
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json_str) {
            let fallback_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
            let tag_name = v["tag_name"].as_str().unwrap_or(&fallback_ver).to_string();
            let body = v["body"].as_str().unwrap_or("Release notes unavailable").to_string();
            let html_url = v["html_url"].as_str().unwrap_or("https://github.com/yunusemreyl/omen-space/releases").to_string();

            let download_url = format!("https://github.com/{}/archive/refs/tags/{}.tar.gz", REPO_OWNER_NAME, tag_name);
            return (tag_name, body, html_url, download_url);
        }
    }

    (
        format!("v{}", CURRENT_VERSION),
        "No update release metadata found".to_string(),
        "https://github.com/yunusemreyl/omen-space/releases".to_string(),
        format!("https://github.com/{}/releases", REPO_OWNER_NAME),
    )
}

fn is_newer_semver(current: &str, remote: &str) -> bool {
    let clean_remote = remote.trim_start_matches('v');
    let parse_ver = |s: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = s.split('.')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect();
        (
            *parts.first().unwrap_or(&0),
            *parts.get(1).unwrap_or(&0),
            *parts.get(2).unwrap_or(&0),
        )
    };

    let c = parse_ver(current);
    let r = parse_ver(clean_remote);

    r > c
}

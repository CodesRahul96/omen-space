use clap::Subcommand;
use anyhow::Result;
use crossterm::style::Stylize;
use crate::dbus_proxy::PlatformProxy;

#[derive(Subcommand, Debug, Clone)]
pub enum OverlayCommand {
    /// Toggle HUD overlay visibility (Shift+F2)
    Toggle,
    /// Launch overlay in background daemon mode
    Daemon,
}

pub async fn handle(cmd: &OverlayCommand, conn: &zbus::Connection) -> Result<()> {
    match cmd {
        OverlayCommand::Toggle => {
            let proxy = PlatformProxy::new(conn).await?;
            let res = proxy.toggle_overlay().await?;
            if res == "OK" {
                println!("{} Overlay toggle signal sent.", "✓".green().bold());
            } else {
                println!("{} Failed to toggle overlay: {}", "✗".red().bold(), res);
            }
        }
        OverlayCommand::Daemon => {
            println!("{} Starting OMEN Overlay in daemon mode...", "🎮".cyan());
            let _ = std::process::Command::new("omen-overlay")
                .arg("--daemon")
                .spawn();
        }
    }
    Ok(())
}

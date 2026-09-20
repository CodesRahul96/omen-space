use clap::Subcommand;
use zbus::Connection;
use anyhow::Result;
use crate::dbus_proxy::PlatformProxy;

#[derive(Subcommand, Debug, Clone)]
pub enum SystemCommand {
    /// Get static hardware and system info
    Info,
    /// Apply hardware keyboard fixes
    FixKeyboard {
        #[arg(long)]
        prtsc: bool,
        #[arg(long)]
        f1: bool,
    },
    /// Set battery charge limit (50 to 100)
    BatteryCare {
        limit: u32,
    },
    /// Run WMI hardware diagnostics suite
    Diagnostics,
    /// Generate a hardware triage bundle for bug reports
    TriageBundle,
    /// Run fan cleaning ritual
    CleanFans,
    /// Check for conflicting daemons
    CheckConflicts,
    /// Check for BIOS updates
    CheckBios,
    /// Check for Omen Space updates
    CheckUpdate,
    /// Apply Omen Space update
    ApplyUpdate,
    /// Clear page cache memory
    CleanMemory,
}

pub async fn handle(cmd: &SystemCommand, conn: &Connection) -> Result<()> {
    let proxy = PlatformProxy::new(conn).await?;

    match cmd {
        SystemCommand::Info => {
            let res = proxy.get_hardware_dump_json().await?;
            println!("{}", res);
        }
        SystemCommand::FixKeyboard { prtsc, f1 } => {
            let res = proxy.set_keyboard_fixes(*prtsc, *f1).await?;
            println!("Response: {}", res);
        }
        SystemCommand::BatteryCare { limit } => {
            let res = proxy.set_battery_care(*limit).await?;
            println!("Response: {}", res);
        }
        SystemCommand::Diagnostics => {
            println!("Running WMI Diagnostics. This may take a few seconds...");
            let res = proxy.run_wmi_diagnostics().await?;
            println!("Report:\n{}", res);
        }
        SystemCommand::TriageBundle => {
            println!("Generating triage bundle...");
            let res = proxy.generate_triage_bundle().await?;
            println!("Archive path: {}", res);
        }
        SystemCommand::CleanFans => {
            println!("Starting fan cleaning routine. Fans will max out for a few seconds.");
            let res = proxy.run_fan_cleaning().await?;
            println!("Response: {}", res);
        }
        SystemCommand::CheckConflicts => {
            let res = proxy.check_conflicts().await?;
            println!("Conflicts:\n{}", res);
        }
        SystemCommand::CheckBios => {
            println!("Checking HP Catalog for BIOS updates...");
            let res = proxy.check_bios_update().await?;
            println!("BIOS Update Info:\n{}", res);
        }
        SystemCommand::CheckUpdate => {
            println!("Checking GitHub for Omen Space updates...");
            let res = proxy.check_app_update().await?;
            println!("Update Info:\n{}", res);
        }
        SystemCommand::ApplyUpdate => {
            println!("Applying Omen Space update...");
            let res = proxy.apply_app_update().await?;
            println!("Response:\n{}", res);
        }
        SystemCommand::CleanMemory => {
            let res = proxy.clean_memory().await?;
            println!("Response: {}", res);
        }
    }

    Ok(())
}

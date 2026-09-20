use serde::{Deserialize, Serialize};
use log::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConflictReport {
    pub has_conflicts: bool,
    pub conflicting_processes: Vec<String>,
    pub warning_message: String,
}

pub struct ConflictDetector;

impl ConflictDetector {
    pub fn check_conflicts() -> ConflictReport {
        let known_conflicts = vec![
            ("nbfc", "Notebook FanControl (nbfc) daemon running — may conflict with HP WMI fan writes"),
            ("throttled", "Lenovo/Generic throttled service running — may race with CPU thermal limits"),
            ("ryzenadj", "Background ryzenadj loop active — may collide with SMU power limit writes"),
            ("hp-health", "Legacy HP Health service active"),
            ("oghaagent", "HP OMEN Gaming Hub background service (Wine/Proton) active"),
        ];

        let running_procs = crate::sysmon::get_running_process_names();
        let mut conflicts_found = Vec::new();

        for (proc_name, desc) in known_conflicts {
            if running_procs.contains(&proc_name.to_lowercase()) {
                conflicts_found.push(format!("{} ({})", proc_name, desc));
            }
        }

        let has_conflicts = !conflicts_found.is_empty();
        let warning_message = if has_conflicts {
            format!("Warning: {} potential conflicting thermal/power process(es) detected.", conflicts_found.len())
        } else {
            "No thermal or EC control software conflicts detected. Coexistence clear.".to_string()
        };

        if has_conflicts {
            info!("{}", warning_message);
        }

        ConflictReport {
            has_conflicts,
            conflicting_processes: conflicts_found,
            warning_message,
        }
    }
}

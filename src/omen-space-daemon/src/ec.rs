#![allow(dead_code)]
#![allow(unused_imports)]
use std::fs::OpenOptions;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use std::process::Command;
use log::{info, warn, debug};

const EC_PATH: &str = "/sys/kernel/debug/ec/ec0/io";

const REG_FAN1_SPEED_PCT: u64 = 0x2E;
const REG_FAN2_SPEED_PCT: u64 = 0x2F;
const REG_FAN1_SPEED_SET: u64 = 0x34;
const REG_FAN2_SPEED_SET: u64 = 0x35;
const REG_CPU_TEMP: u64 = 0x57;
const REG_BIOS_CONTROL: u64 = 0x62;
const REG_TIMER: u64 = 0x63;
const REG_PERF_MODE: u64 = 0x95;
const REG_GPU_TEMP: u64 = 0xB7;
const REG_THERMAL_POWER: u64 = 0xBA;
const REG_FAN_BOOST: u64 = 0xEC;
const REG_FAN_STATE: u64 = 0xF4;

const UNSAFE_MODELS: &[&str] = &["16t-ah0", "16-ah0", "16-ap0", "17t-ah0", "17-ah0", "transcend 14"];
const UNSAFE_BOARDS: &[&str] = &["8c58", "8d24"];

pub struct LinuxEcController {
    has_ec_access: bool,
    board_id: String,
    is_unsafe_ec_model: bool,
    is_unsafe_model: bool,
}

impl LinuxEcController {
    pub fn new() -> Self {
        let board_id = std::fs::read_to_string("/sys/class/dmi/id/board_name")
            .unwrap_or_else(|_| "UNKNOWN".to_string())
            .trim()
            .to_string();

        let is_unsafe_model = Self::check_unsafe_model();
        if is_unsafe_model {
            warn!("LinuxEcController: UNSAFE MODEL DETECTED! Legacy EC writes will be blocked to prevent Caps Lock panic.");
        }

        let mut ec = LinuxEcController {
            has_ec_access: false,
            board_id,
            is_unsafe_ec_model: false,
            is_unsafe_model,
        };
        ec.ensure_ec_sys();
        ec.has_ec_access = Path::new(EC_PATH).exists();
        ec
    }

    fn check_unsafe_model() -> bool {
        let product_name = std::fs::read_to_string("/sys/class/dmi/id/product_name").unwrap_or_default().to_lowercase();
        let board_name = std::fs::read_to_string("/sys/class/dmi/id/board_name").unwrap_or_default().to_lowercase();

        for model in UNSAFE_MODELS {
            if product_name.contains(model) {
                return true;
            }
        }
        for board in UNSAFE_BOARDS {
            if board_name.contains(board) {
                return true;
            }
        }
        false
    }

    pub fn needs_ec_fallback(&self) -> bool {
        self.board_id == "8E35" || self.board_id == "8A43" || self.board_id == "8A42"
    }

    pub fn has_ec_access(&self) -> bool {
        self.has_ec_access
    }

    fn ensure_ec_sys(&self) {
        if Path::new(EC_PATH).exists() {
            return;
        }

        let debugfs_base = "/sys/kernel/debug";
        if !Path::new(debugfs_base).exists() {
            let _ = Command::new("mount")
                .args(["-t", "debugfs", "none", debugfs_base])
                .output();
        }

        let _ = Command::new("modprobe")
            .args(["ec_sys", "write_support=1"])
            .output();
    }

    pub fn try_lazy_ec_load(&mut self) -> bool {
        if self.has_ec_access {
            return true;
        }
        self.ensure_ec_sys();
        self.has_ec_access = Path::new(EC_PATH).exists();
        if self.has_ec_access {
            info!("Lazy EC load succeeded for board {}", self.board_id);
        }
        self.has_ec_access
    }

    pub fn read_byte(&mut self, reg: u64) -> u8 {
        if !self.has_ec_access {
            return 0;
        }
        if self.is_unsafe_ec_model && reg != 0x59 && reg != REG_PERF_MODE {
            return 0;
        }

        match OpenOptions::new().read(true).open(EC_PATH) {
            Ok(mut file) => {
                if file.seek(SeekFrom::Start(reg)).is_ok() {
                    let mut buf = [0u8; 1];
                    if let Ok(n) = file.read(&mut buf) {
                        if n > 0 {
                            return buf[0];
                        }
                    }
                }
            }
            Err(_) => {
                debug!("EC read_byte failed at 0x{:02X}. Kernel lockdown?", reg);
                self.has_ec_access = false;
            }
        }
        0
    }

    pub fn write_byte(&mut self, reg: u64, value: u8) -> bool {
        if self.is_unsafe_model {
            warn!("EC Blocked: Attempted to write 0x{:X} to register 0x{:X} on an unsafe model (2025/Transcend)", value, reg);
            return false;
        }
        if !self.has_ec_access {
            return false;
        }
        if self.is_unsafe_ec_model && reg != 0x59 && reg != REG_PERF_MODE {
            return false;
        }

        match OpenOptions::new().read(true).write(true).open(EC_PATH) {
            Ok(mut file) => {
                if file.seek(SeekFrom::Start(reg)).is_ok()
                    && file.write(&[value]).is_ok() {
                        let _ = file.flush();
                        debug!("EC write_byte success at 0x{:02X} = 0x{:02X}", reg, value);
                        return true;
                    }
            }
            Err(_) => {
                warn!("EC access lost on write. Kernel lockdown?");
                self.has_ec_access = false;
            }
        }
        false
    }

    pub fn get_cpu_temp(&mut self) -> f64 {
        // Read from hwmon instead of EC to prevent conflicts
        // See sysmon.rs for proper HWMon reading. This is deprecated.
        50.0
    }

    pub fn get_gpu_temp(&mut self) -> f64 {
        // Deprecated EC read
        50.0
    }

    pub fn set_fan_speed_pct(&mut self, fan_idx: u32, pct: u32) -> bool {
        if self.is_unsafe_ec_model || self.is_unsafe_model || !self.has_ec_access {
            return false;
        }
        let pct = pct.clamp(0, 100) as u8;
        let reg = if fan_idx == 1 { REG_FAN1_SPEED_PCT } else { REG_FAN2_SPEED_PCT };
        self.write_byte(reg, pct)
    }

    pub fn set_fan_boost(&mut self, enable: bool) -> bool {
        if self.is_unsafe_ec_model || !self.has_ec_access {
            return false;
        }
        let val = if enable { 0x0C } else { 0x00 };
        self.write_byte(REG_FAN_BOOST, val)
    }

    pub async fn restore_auto_mode(&mut self) -> bool {
        if !self.has_ec_access {
            return false;
        }

        self.write_byte(REG_FAN1_SPEED_SET, 0x00);
        self.write_byte(REG_FAN2_SPEED_SET, 0x00);
        self.write_byte(REG_FAN1_SPEED_PCT, 0x00);
        self.write_byte(REG_FAN2_SPEED_PCT, 0x00);
        
        self.write_byte(REG_FAN_BOOST, 0x00);
        
        if !self.write_byte(REG_FAN_STATE, 0x00) {
            return false;
        }
            
        if !self.write_byte(REG_BIOS_CONTROL, 0x00) {
            return false;
        }
        
        self.write_byte(REG_TIMER, 0x78);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        self.write_byte(REG_FAN_STATE, 0x00);
        
        true
    }

    pub fn set_manual_fan_speed(&mut self, pct: u8) -> bool {
        if !self.has_ec_access {
            return false;
        }
        
        self.write_byte(REG_BIOS_CONTROL, 0x06);
        self.write_byte(REG_FAN_STATE, 0x02);
        
        let pct = pct.clamp(0, 100);
        self.write_byte(REG_FAN1_SPEED_PCT, pct);
        self.write_byte(REG_FAN2_SPEED_PCT, pct);
        
        true
    }

    pub fn set_perf_mode(&mut self, mode: &str) -> bool {
        if !self.has_ec_access {
            return false;
        }

        // EC Fallback for older boards that don't fully support WMI platform_profile
        if self.board_id == "8E35" || self.board_id == "8A43" || self.board_id == "8A42" {
            let val = match mode.to_lowercase().as_str() {
                "performance" | "max" => 0x31,
                "cool" | "eco" => 0x50,
                _ => 0x30,
            };
            info!("Using EC Fallback for {} to set thermal profile: 0x{:02X} at 0x59", self.board_id, val);
            return self.write_byte(0x59, val);
        }

        if self.is_unsafe_ec_model {
            return false;
        }

        let val = match mode.to_lowercase().as_str() {
            "performance" | "max" => 0x31,
            "cool" | "eco" => 0x50,
            _ => 0x30,
        };
        self.write_byte(REG_PERF_MODE, val)
    }
}

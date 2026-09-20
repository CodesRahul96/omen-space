# OMENSpace Backend Daemon (`omen-space-daemon`)

The backend daemon is the absolute core of the OMENSpace stack. Because controlling laptop hardware (like fan speeds, CPU power limits, and RGB memory registers) requires strict `root` privileges, the daemon is designed to run in the background as a `systemd` service and expose safe methods over D-Bus for the user interface to interact with.

## Responsibilities

1. **Hardware Interfacing:**
   - **WMI / ACPI:** Communicates with HP's ACPI endpoints to change performance modes (Eco, Balanced, Performance).
   - **Embedded Controller (EC):** Reads and writes directly to EC memory to control fan speeds (Auto, Max, or Manual dynamic curves).
   - **MSR (Model-Specific Registers):** Interacts with Intel CPUs directly to apply undervolting and TCC (Thermal Control Circuit) offset limits.
   - **Sysfs & RAPL:** Manages Intel Power Limits (PL1/PL2) directly through `/sys/class/powercap`.
   - **NVIDIA-SMI:** Automatically limits the GPU TGP (Total Graphics Power) when switching modes.

2. **D-Bus Server Setup (`org.hp.omen.*`)**
   - The daemon uses the `zbus` Rust crate to bind to the Linux System Bus.
   - It exposes multiple objects/interfaces (e.g., `org.hp.omen.fans`, `org.hp.omen.power`, `org.hp.omen.lighting`).
   - A Polkit/DBus security configuration (`data/org.hp.omen.conf`) ensures that only users in the `omen-hw` or `wheel` group can send messages to these endpoints, preventing random unprivileged apps from altering hardware states.

## Key Files

- `src/main.rs`: Entry point. Initializes asynchronous tasks and mounts the D-Bus server.
- `src/fans.rs`: EC logic for reading RPMs and injecting custom PWM curves.
- `src/power.rs`: Manages TDP/TGP and Undervolting.
- `src/lighting.rs`: Parses colors and animations, pushing raw byte packets to the kernel driver.

## Security Considerations
Since this daemon runs as `root`, it rigorously parses incoming D-Bus arguments to ensure no malicious commands are executed (e.g., strictly parsing integer limits for power rather than executing raw shell strings).

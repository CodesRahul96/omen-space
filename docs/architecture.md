# OMENSpace Architecture Map & Documentation

This document serves as the technical map for the **OMENSpace** project. It outlines how the software interacts with the hardware, the inter-process communication mechanisms, and the layout of the source code.

## 1. High-Level Architecture

OMENSpace follows a **Client-Server Architecture** operating locally on the user's Linux machine via **D-Bus**.

```mermaid
graph TD
    subgraph "Hardware & Kernel"
        K[Linux Kernel]
        ACPI[HP WMI / ACPI]
        MSR[MSR Registers]
        DKMS[omen-rgb-keyboard DKMS]
    end

    subgraph "Root Context (Server)"
        Daemon[omen-space-daemon]
    end

    subgraph "User Context (Clients)"
        GUI[omen-gui (GTK4)]
        CLI[omen-cli]
        Tray[omen-tray]
    end
    
    Daemon <-->|Sysfs / Ioctl| K
    Daemon <-->|WMI Calls| ACPI
    Daemon <-->|CPU Control| MSR
    Daemon <-->|RGB Data| DKMS
    
    GUI <-->|D-Bus (org.hp.omen.*)| Daemon
    CLI <-->|D-Bus (org.hp.omen.*)| Daemon
    Tray <-->|D-Bus (org.hp.omen.*)| Daemon
```

### Why this architecture?
Direct hardware manipulation (changing fan curves, editing CPU MSR registers, modifying WMI endpoints) requires `root` access. 
By placing all hardware logic inside `omen-space-daemon` (which runs as a root systemd service) and having it expose a safe D-Bus API, client applications like `omen-gui` can run completely unprivileged. This aligns with modern Linux security standards (similar to how NetworkManager or systemd-logind works).

---

## 2. Component Map

The repository is organized into specific directories representing the components.

### 2.1. `src/omen-space-daemon/`
The core backend service.
- **Language:** Rust (tokio asynchronous runtime)
- **Role:** Handles all logic. Reads sensors, modifies power limits (RAPL/NVML), sets fan speeds, and pushes RGB data.
- **Key Files:**
  - `main.rs`: Entry point, initializes the DBus server via `zbus`.
  - `power.rs`: Handles CPU (PL1/PL2, Undervolt via MSR) and GPU (NVIDIA-SMI / TGP limits).
  - `fans.rs`: Interfaces with HP's EC (Embedded Controller) to set manual or dynamic fan curves.
  - `lighting.rs`: Pushes raw byte payloads to the kernel driver for RGB zones/keys.

### 2.2. `src/omen-gui/`
The primary user interface.
- **Language:** Rust
- **Framework:** GTK4 + LibAdwaita
- **Role:** Presents a beautiful, reactive desktop interface for the user to configure their hardware.
- **Key Files:**
  - `main.rs`: Window initialization, CSS loading, layout structure.
  - `daemon_client.rs`: Contains the `zbus` proxy macros that generate safe Rust methods to talk to the DBus API.
  - `performance_control.rs`: UI for thermal profiles (Eco/Balanced/Performance) and Fan Modes (Auto/Max/Custom).
  - `keyboardrgb.rs`: UI for selecting colors and effects.
  - `updater.rs`: Implements GitHub API checking for OTA software updates and `fwupdmgr` for BIOS updates.
  - `asset_resolver.rs`: Ensures `.svg` and `.png` images are loaded from the correct system paths (`/usr/share/omen-space/assets/`).

### 2.3. `src/omen-cli/`
The command-line tool.
- **Language:** Rust
- **Role:** Allows scripts or power users to control hardware directly from the terminal (e.g., `omen-cli fans max`).

### 2.4. `src/omen-tray/`
The system tray icon.
- **Language:** Rust
- **Role:** Runs quietly in the background, providing quick toggles (right-click menu) without needing to open the full GTK app.

### 2.5. `driver/`
The kernel module.
- **Language:** C
- **Role:** Some newer HP keyboards (especially per-key RGB) require a custom Linux kernel module because the mainline kernel lacks support. This folder contains the DKMS driver that creates character devices the daemon can write to.

### 2.6. `data/`
System integration files.
- `org.hp.omen.conf`: Polkit / D-Bus security policy allowing standard users to communicate with the root daemon.
- `omen-space-daemon.service`: The systemd service definition.
- `org.hp.OmenSpace.desktop`: The application launcher for Desktop Environments (GNOME, KDE).
- `99-omen-space.rules`: Udev rules to ensure devices have correct permissions.

---

## 3. Data Flow Example: Setting a Fan Curve

To understand how the app works, here is the lifecycle of a user action:

1. **User Action:** The user clicks a custom fan preset button in `omen-gui`.
2. **GUI Layer:** `performance_control.rs` detects the click and calls `crate::daemon_client::set_fan_mode_async("custom", curve_data)`.
3. **D-Bus Layer:** The `zbus` library serializes this call and sends it over the Linux System Bus to the `org.hp.omen.fans` interface.
4. **Daemon Layer:** `omen-space-daemon` receives the message.
5. **Hardware Layer:** The daemon translates the `curve_data` into specific hex bytes and writes them to the HP WMI ACPI endpoint or the Embedded Controller (EC) memory registers.
6. **Hardware Response:** The fans immediately spin up to the requested curve.

## 4. Build & Install System
The `setup.sh` script automates compilation using `cargo build --release` for all 4 Rust crates, safely removes old legacy (`omenctl`) installations, copies binaries to `/usr/bin/`, installs the DKMS module, and restarts systemd services.

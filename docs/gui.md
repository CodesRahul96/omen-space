# OMENSpace GUI (`omen-gui`)

The graphical user interface for OMENSpace is designed to be modern, responsive, and visually cohesive with Linux desktop environments (like GNOME). It is entirely built in **Rust** using **GTK4** and **LibAdwaita**.

## Responsibilities

1. **User Interaction:**
   - Provides an intuitive dashboard for users to monitor hardware statistics (CPU/GPU temperatures, RPMs).
   - Allows users to easily toggle Fan Modes, Performance Profiles, and RGB animations without using a terminal.
2. **D-Bus Client Implementation:**
   - The GUI has **no root privileges**. It cannot control hardware directly.
   - It uses `zbus` proxy macros (inside `daemon_client.rs`) to asynchronously send commands to the `omen-space-daemon`.
3. **Asset & Theme Management:**
   - Automatically loads CSS styling (`style.css`) for custom UI components (like the custom toggle chips).
   - Resolves image paths dynamically (`asset_resolver.rs`) so the application works perfectly whether it is launched locally (`cargo run`) or installed system-wide (`/usr/share/omen-space/assets`).
4. **App Updator:**
   - Connects to the GitHub API to check for software updates.
   - Triggers `fwupdmgr` to scan for HP BIOS and firmware updates natively.

## Key Files

- `src/main.rs`: Window initialization, sidebar navigation, and CSS loading.
- `src/performance_control.rs`: The logic and UI for the "Power & Fans" page. Handles the dynamic creation of custom fan curve presets.
- `src/keyboardrgb.rs`: The RGB color picker and animation effect selector.
- `src/monitoring.rs`: Renders the real-time statistics cards (CPU/GPU temps).
- `src/updater.rs`: Manages OTA update checks.
- `src/i18n.rs`: Handles localization/translations (e.g., Turkish and English support).

# OMENSpace Tray (`omen-tray`)

The `omen-tray` crate provides a lightweight, unobtrusive system tray (AppIndicator/KDE System Notification Item) icon for OMENSpace. It is meant to run continuously in the background without eating up memory or CPU cycles.

## Responsibilities & Features

1. **Background Presence:**
   - Runs independently of the main GUI application. 
   - Uses the `ksni` crate to register itself seamlessly in the system tray of modern Desktop Environments like GNOME (via AppIndicator extension), KDE Plasma, XFCE, and Sway.
2. **Quick Actions Context Menu:**
   - **Launch GUI:** Fast shortcut to spawn the full `omen-gui` application.
   - **Performance Modes:** Right-click to quickly toggle between `Performance`, `Balanced`, and `Eco` modes.
   - **Fan Modes:** Quick toggle for `Auto` and `Max` fan modes.
   - **Exit:** Gracefully terminates the tray applet.
3. **D-Bus Integration:**
   - Just like `omen-gui` and `omen-cli`, the tray applet is entirely unprivileged.
   - It asynchronously sends `zbus` messages to the root `omen-space-daemon` over the D-Bus (`org.hp.omen.*` interfaces).

## Technical Architecture

### Core Libraries
- **`ksni`**: A Rust library that implements the StatusNotifierItem (SNI) protocol. This is the modern standard for Linux system trays, replacing legacy X11 embedded icons.
- **`zbus`**: Used to communicate with the DBus daemon.
- **`tokio`**: The asynchronous runtime. Even though the tray is a simple app, it uses `tokio` to asynchronously send DBus messages without blocking the GUI/Tray event loop.

### Code Breakdown (`src/main.rs`)

1. **`struct Tray`**: 
   The core structure implementing the `ksni::Tray` trait. It defines the icon (`omen-space`) and the title.
2. **`fn menu(&self)`**:
   This trait method constructs the actual drop-down menu hierarchy:
   - It uses `StandardItem` for clickable buttons and `SubMenu` for nested categories (e.g., "Güç Profili").
   - When an item is clicked, its `activate` closure is fired. For DBus calls, it spawns a detached `tokio::spawn` task so the tray menu closes instantly while the command executes in the background.
3. **`get_conn()`**:
   A helper function that attempts to connect to the DBus. It defaults to the System bus, but falls back to the Session bus if needed for testing.
4. **`set_power_profile()` & `set_fan_mode()`**:
   Helper methods that execute `conn.call_method()` targeting `/org/hp/omen/Power` and `/org/hp/omen/Fan` endpoints respectively.
5. **Main Event Loop (`main()`)**:
   Initializes the logger, creates the `ksni::TrayService`, spawns it, and then enters an infinite `tokio::time::sleep` loop to keep the process alive while the SNI server runs in a background thread.

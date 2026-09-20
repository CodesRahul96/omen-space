# OMENSpace CLI (`omen-cli`)

The `omen-cli` crate provides a command-line interface for power users, script writers, and headless environments. 

## Responsibilities

1. **Terminal Control:**
   - Exposes every feature available in the GUI directly to the terminal.
   - Designed to be extremely fast. It executes commands and exits immediately, making it perfect for custom shell scripts, shortcuts, or keybindings (e.g., binding a keyboard shortcut to `omen-cli fans max`).
2. **D-Bus Communication:**
   - Like all client components in OMENSpace, the CLI runs completely unprivileged. 
   - It acts as a synchronous/asynchronous DBus client, forwarding arguments to the `omen-space-daemon`.

## Common Usage Examples
*(Note: API is subject to change based on actual CLI implementation)*

- `omen-cli power performance` - Sets the thermal and power limits to Performance mode.
- `omen-cli fans auto` - Returns fan control to the system BIOS.
- `omen-cli rgb static #FF0000` - Sets keyboard backlight to solid red.
- `omen-cli system info` - Retrieves real-time thermal diagnostics and current daemon status.

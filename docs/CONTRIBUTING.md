# Contributing to OMENSpace

Thank you for your interest in contributing to OMENSpace! As an open-source tool aiming to provide the best HP Omen/Victus hardware control on Linux, we welcome all pull requests—from typo fixes to entirely new hardware reverse-engineering.

## Development Environment Setup

1. **Prerequisites:**
   - Rust toolchain (via `rustup`)
   - `libgtk-4-dev` and `libadwaita-1-dev` (for `omen-gui`)
   - `dbus` and `pkg-config`
   - Kernel headers (for building the DKMS module)

2. **Local Compilation:**
   Instead of using `sudo ./setup.sh`, you can build components locally.
   ```bash
   cd src/omen-space-daemon
   cargo build
   
   cd ../omen-gui
   cargo run
   ```

## Architectural Rules for Contributors

1. **No Root in the GUI:**
   - OMENSpace strictly follows a split privilege model. `omen-gui`, `omen-cli`, and `omen-tray` must **never** require `sudo`. 
   - If you need to access a new `/sys/` or `/dev/` endpoint, that logic MUST be written in `omen-space-daemon`.
   - The GUI will communicate with the daemon via D-Bus (`zbus`).

2. **D-Bus Interface Definitions:**
   - Ensure you update the XML definitions in `data/org.hp.omen.conf` if you add new D-Bus methods, to ensure standard users in the `omen-hw` group can access them.

3. **Asynchronous Code (Tokio):**
   - The GUI thread (GTK) must never be blocked. When calling the D-Bus daemon from `omen-gui` or `omen-tray`, you must use `tokio::spawn` or `glib::spawn_future_local`.
   - Avoid `std::thread::sleep`—use `tokio::time::sleep`.

## Code Style (Rust)
- Run `cargo fmt` before submitting your PR.
- Run `cargo clippy -- -D warnings` to ensure there are no linting issues.
- Keep variable names descriptive. Use `snake_case` for variables/functions and `CamelCase` for structs/enums.

## Submitting a Pull Request
1. Fork the repository.
2. Create a new branch: `git checkout -b feature/my-cool-feature`.
3. Commit your changes logically with descriptive commit messages.
4. Push to your branch and open a PR against the `main` branch.
5. In your PR description, explain **what** you changed and **why**.

# Comprehensive Code Review Guide for OMENSpace

This guide is intended for maintainers, senior contributors, and reviewers evaluating Pull Requests (PRs) submitted to the OMENSpace project. Our primary objective is to maintain a rock-solid, secure, and memory-safe architecture while interacting with sensitive laptop hardware.

---

## 1. Security & Privilege Boundaries (CRITICAL)

OMENSpace uses a split-privilege architecture. The user-facing apps (`omen-gui`, `omen-cli`, `omen-tray`) run entirely unprivileged, while the `omen-space-daemon` runs as root to interact with the kernel and hardware.

- **Zero Sudo in User Space:** 
  If a PR modifies `omen-gui`, `omen-tray`, or `omen-cli` to execute shell commands with `sudo` (e.g., `pkexec` or `sudo systemctl`), **reject it immediately**. All hardware logic must be routed through D-Bus to the daemon.
- **Strict D-Bus Input Validation:**
  When reviewing `omen-space-daemon`, ensure that incoming arguments from D-Bus clients are rigorously sanitized.
  - **No Shell Injection:** Never pass raw D-Bus strings into `tokio::process::Command` without strict escaping or parsing.
  - **Type Checking:** Ensure strings representing profiles (e.g., `"eco"`, `"balanced"`) are mapped to internal Enums using `match` statements with a fallback error case.
- **Polkit & DBus Policies:**
  If a PR adds a new D-Bus method to `src/omen-space-daemon`, ensure the policy file `data/org.hp.omen.conf` is updated correctly. Access should remain restricted to the `omen-hw` group.

---

## 2. Asynchronous Execution & Concurrency

OMENSpace relies on the `tokio` asynchronous runtime to keep both the daemon and the GUI highly responsive.

- **GTK Event Loop Freezing:** 
  In the GUI (`omen-gui`), heavy DBus calls or I/O operations **must not block the main thread**. Look for `glib::spawn_future_local` or `tokio::spawn`. If you spot `std::thread::sleep`, synchronous `std::fs::read_to_string` for large files, or `reqwest::blocking` inside UI signal handlers, request the author to switch to asynchronous equivalents.
- **Memory Leaks in GTK Signals:**
  When closures are attached to GTK buttons (e.g., `connect_clicked`), ensure the author correctly uses the `glib::clone!(@weak self)` macro. Capturing strong references inside signal closures will create reference cycles and leak memory.
- **Daemon Concurrency:** 
  Hardware access (like reading slow I2C buses, writing to Sysfs, or running `nvidia-smi`) can block the thread. Ensure these operations are spawned on dedicated blocking threads (`tokio::task::spawn_blocking`) if they take more than a few milliseconds, so they don't block the main DBus event loop.

---

## 3. Hardware Safety (ACPI, EC, MSR)

HP hardware is highly proprietary and sensitive. Incorrect writes can brick devices, cause thermal shutdowns, or freeze the OS.

- **Embedded Controller (EC) Writes:** 
  Review changes to `fans.rs` meticulously. Ensure that magic bytes, memory offsets, and PWM duty cycles match verified HP documentation or known omencore reverse-engineering.
- **Bounds Checking for Power:** 
  If a PR modifies RAPL (PL1/PL2) limits or MSR undervolting, verify that the daemon enforces hard limits (e.g., preventing a user from sending a D-Bus message that sets a 200W limit on a 45W CPU).
- **DKMS Module Updates (`driver/`):** 
  If the C kernel module is modified:
  - Check for proper memory allocation and `kfree()` to prevent kernel panics.
  - Prevent null pointer dereferences.
  - Ensure the module hooks correctly into the Linux USB/I2C HID subsystems without conflicting with mainline modules.

---

## 4. Rust Idioms and Standards

- **Clippy and Fmt:** 
  All PRs must pass `cargo clippy -- -D warnings` and `cargo fmt`. Reviewers should not manually point out missing commas or standard linting errors. Ask the contributor to fix CI/CD pipeline failures before reviewing.
- **Error Handling (No Panics):** 
  Avoid `unwrap()`, `expect()`, or `panic!()` in production code. A failure to read a temperature sensor should never crash the entire daemon.
  - **In the Daemon:** Return a generic `Result` and log errors via `log::error!`.
  - **In the GUI:** Present a user-friendly GTK dialog (e.g., `adw::MessageDialog`) or a toast notification.
- **Unsafe Code:** 
  Any `unsafe {}` blocks must be highly scrutinized. The author must include a `// SAFETY:` comment explaining exactly why the unsafe block is sound. In general, unsafe code should only exist when interacting directly with C APIs (like GTK or libc ioctls).

---

## 5. D-Bus API Design

- **Backward Compatibility:** 
  Avoid breaking existing D-Bus signatures if possible. If a method signature changes (e.g., adding an argument), ensure the GUI, CLI, and Tray clients are all updated in the same PR.
- **Stateless Daemon:** 
  The daemon should remain as stateless as possible. It should read the current hardware state directly from the kernel/ACPI rather than caching state internally, as the hardware state can be changed by the BIOS or OS outside of OMENSpace.

---

## 6. Review Checklist for Maintainers

Before merging a PR, explicitly verify the following:

- [ ] **Security:** Does this break the unprivileged GUI model? Are inputs sanitized?
- [ ] **Performance:** Is GTK blocking avoided? Are slow operations offloaded to async tasks?
- [ ] **Memory Safety:** Are GTK signal closures using weak clones? Is `unsafe` justified?
- [ ] **Error Handling:** Are errors caught gracefully without panicking the application?
- [ ] **Hardware Limits:** Are new power/fan commands bounded by safe hardcoded limits?
- [ ] **Formatting:** Does `cargo check` and `cargo clippy` pass?
- [ ] **Documentation:** If a new feature was added, was `README.md` or the `docs/` folder updated?

# Kernel Driver (`omen-rgb-keyboard` DKMS)

The `driver/` directory contains a C-based Linux kernel module. 

## Why is a kernel module needed?
For many functions (like fan control, CPU power, and reading thermal sensors), OMENSpace interacts with standard Linux kernel interfaces (like `sysfs`, `hwmon`, and standard WMI-ACPI events). 

However, modern HP Omen and Victus laptops utilize proprietary, undocumented I2C/USB endpoints for **Keyboard RGB Lighting** (especially 4-zone and per-key RGB models). The mainline Linux kernel (`hid-hp`) does not fully support these advanced lighting controls. 

## Responsibilities
1. **Hardware Communication:** Sends the precise hexadecimal byte payloads required by the keyboard's microcontroller to change colors and animations.
2. **Exposing a Character Device:** The driver mounts a node (typically in `/dev/`) or a specific `sysfs` tree that allows user-space applications to write color data.
3. **DKMS Integration:** 
   - Distributed as a DKMS (Dynamic Kernel Module Support) package.
   - This ensures that whenever the user updates their Linux kernel (e.g., from `6.1` to `6.5`), the driver automatically recompiles itself for the new kernel version during boot.

## How it interacts with OMENSpace
The `omen-space-daemon` contains a `lighting.rs` module. When the user selects a color in the GUI, the daemon translates this color into a raw byte buffer and pipes it directly into the kernel module's exposed endpoint, triggering an instant hardware-level color change on the keyboard.

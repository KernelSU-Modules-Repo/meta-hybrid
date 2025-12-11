# Meta-Hybrid Mount

![Language](https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/Platform-Android-green?style=flat-square&logo=android)
![License](https://img.shields.io/badge/License-GPL--3.0-blue?style=flat-square)

**Meta-Hybrid Mount** is a next-generation hybrid mount metamodule designed for KernelSU. Written in native Rust, it intelligently combines **OverlayFS** and **Magic Mount** technologies to provide a more efficient, stable, and stealthy module management experience compared to traditional mounting solutions.

This project includes a modern WebUI management interface built with Svelte, allowing users to monitor status, manage module modes, and view logs in real-time.

**[ 🇨🇳 中文 (Chinese) ](README_ZH.md)**

---

## ✨ Core Features

### 🚀 True Hybrid Engine
* **Smart Strategy**: Prioritizes **OverlayFS** to achieve optimal I/O performance and filesystem merging capabilities.
* **Automatic Fallback**: Automatically and seamlessly falls back to the **Magic Mount** mechanism when OverlayFS mounting fails, the target is unsupported, or when forcibly specified by the user.
* **Rust Native**: The core daemon is written in Rust, utilizing `rustix` for direct system calls, ensuring safety and high efficiency.

### 🔄 Smart Sync
* **Fast Boot**: Abandons the inefficient pattern of full copying on every boot. The daemon compares `module.prop` checksums and only synchronizes new or modified modules.
* **I/O Optimization**: Drastically reduces disk I/O usage during boot, significantly improving system startup speed.

### 💾 Smart Storage
* **Tmpfs Priority**: Defaults to attempting to use **Tmpfs** (memory-based filesystem) as the storage backend. It offers extremely fast read/write speeds and is cleared on reboot, providing high stealth.
* **Automatic Image Fallback**: Automatically detects if the environment supports XATTR (required for SELinux). If Tmpfs does not support it, the system automatically creates and mounts a 2GB `ext4` loop image (`modules.img`) and includes capability for automatic image repair.

### 🐾 Stealth Mode (Paw Pad / Nuke)
* **Sysfs Cleanup**: Supports removing KernelSU traces in Sysfs via `ioctl` operations to enhance the stealth of the Root environment.

### 📱 Modern WebUI
* Built-in management panel based on Svelte + Vite.
* Supports Dark/Light theme switching and multiple languages (Chinese, English, Japanese, Russian, Spanish).
* Real-time monitoring of storage usage, mount status, and system logs.

---

## 🛠️ Architecture

The workflow of Meta-Hybrid Mount is as follows:

1.  **Environment Init**: Initialize logging and camouflage the process name as `kworker`.
2.  **Storage Prep**: Attempt to mount Tmpfs; if it fails or lacks extended attribute support, mount/repair `modules.img`.
3.  **Inventory Scan**: Scan the module directory and read module configurations and modes (Auto/Magic).
4.  **Incremental Sync**: Synchronize changed module files to the runtime storage directory.
5.  **Mount Planning**:
    * Generate the OverlayFS hierarchy (Lowerdirs).
    * Identify paths requiring Magic Mount.
6.  **Execution**: Execute mount operations according to the plan. If an Overlay fails, the module is automatically added to the Magic Mount queue for retry.
7.  **State Save**: Save runtime state for the WebUI to read.

---

## ⚙️ Configuration

The configuration file is located at `/data/adb/meta-hybrid/config.toml`. You can also modify it visually via the WebUI.

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `moduledir` | String | `/data/adb/modules/` | Path to the module source directory. |
| `tempdir` | String | (Auto) | Temporary working directory. Automatically selected if left empty. |
| `mountsource` | String | `KSU` | Mount source name, used for the `source` parameter in OverlayFS. |
| `verbose` | Bool | `false` | Whether to enable detailed debug logging. |
| `partitions` | Array | `[]` | List of extra partitions to mount (besides built-in ones like system/vendor). |
| `force_ext4` | Bool | `false` | Force usage of `modules.img` without attempting Tmpfs. |
| `enable_nuke` | Bool | `false` | Enable "Paw Pad" mode (Clean up Sysfs traces). |
| `disable_umount` | Bool | `false` | Disable add_try_umount |

---

## 🖥️ WebUI Features

After installing the module, you can access the WebUI via the KernelSU manager (or by opening the corresponding address in a browser).

* **Status**:
    * View storage usage of `modules.img` or Tmpfs.
    * View Kernel version, SELinux status, and active mount partitions.
    * Statistics for OverlayFS vs Magic Mount modules.
* **Config**:
    * Visual editor for `config.toml`.
    * One-click configuration reload.
* **Modules**:
    * Search and filter installed modules.
    * **Mode Switching**: Forcibly specify "OverlayFS" or "Magic Mount" mode for specific modules (useful for resolving bootloops caused by specific modules).
* **Logs**:
    * Real-time view of daemon logs (`daemon.log`).
    * Support for log level filtering and searching.

---

## 🔨 Build Guide

This project uses the Rust `xtask` pattern for building and integrates the WebUI build process.

### Prerequisites
* **Rust**: Nightly toolchain (Recommended to use `rustup`)
* **Android NDK**: Version r27+
* **Node.js**: v20+ (For building WebUI)
* **Java**: JDK 17 (For environment configuration)

### Build Commands

1.  **Clone Repository**
    ```bash
    git clone --recursive [https://github.com/YuzakiKokuban/meta-hybrid_mount.git](https://github.com/YuzakiKokuban/meta-hybrid_mount.git)
    cd meta-hybrid_mount
    ```

2.  **Execute Build**
    Use `xtask` to automatically handle WebUI compilation, Rust cross-compilation, and Zip packaging:
    ```bash
    # Build Release version (Includes WebUI and binaries for all architectures)
    cargo run -p xtask -- build --release
    ```

    The build artifacts will be located in the `output/` directory.

3.  **Build Binaries Only (Skip WebUI)**
    If you only modified Rust code, you can skip the WebUI build to save time:
    ```bash
    cargo run -p xtask -- build --release --skip-webui
    ```

### Supported Architectures
The build script compiles the following architectures by default:
* `aarch64-linux-android` (arm64)
* `x86_64-linux-android` (x64)
* `riscv64-linux-android` (riscv64)

---

## 🤝 Contributions & Credits

* Thanks to all contributors in the open-source community.
* Our sister project [Hymo](https://github.com/Anatdx/hymo)
* This project utilizes excellent open-source libraries such as `rustix`, `clap`, `serde`, and `svelte`.

## 📄 License

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0)**. See the [LICENSE](LICENSE) file for details.

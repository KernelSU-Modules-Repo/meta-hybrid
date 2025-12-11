# Meta-Hybrid Mount

![Language](https://img.shields.io/badge/Language-Rust-orange?style=flat-square&logo=rust)
![Platform](https://img.shields.io/badge/Platform-Android-green?style=flat-square&logo=android)
![License](https://img.shields.io/badge/License-GPL--3.0-blue?style=flat-square)

**Meta-Hybrid Mount** 是一个专为 KernelSU 设计的下一代混合挂载（Hybrid Mount）元模块。它采用原生 Rust 编写，通过智能结合 OverlayFS 和 Magic Mount 技术，旨在提供比传统挂载方案更高效、更稳定且更具隐蔽性的模块管理体验。

本项目包含一个基于 Svelte 的现代化 WebUI 管理界面，方便用户实时监控状态、管理模块模式及查看日志。

**[ 🇺🇸 English ](README.md)**

---

## ✨ 核心特性

### 🚀 混合挂载引擎 (True Hybrid Engine)
* **智能策略**：优先使用 **OverlayFS** 以获得最佳的 I/O 性能和文件系统合并能力。
* **自动回退**：当 OverlayFS 挂载失败、目标不支持或用户强制指定时，自动无缝回退到 **Magic Mount** 机制，确保最大兼容性。
* **Rust 原生**：核心守护进程使用 Rust 编写，利用 `rustix` 进行直接系统调用，安全且高效。

### 🔄 智能增量同步 (Smart Sync)
* **极速启动**：摒弃了每次开机全量复制的低效模式。守护进程会对比 `module.prop`，仅同步新增或发生变化的模块。
* **I/O 优化**：大幅减少开机时的磁盘 I/O 占用，显著提升系统启动速度。

### 💾 智能存储后端 (Smart Storage)
* **Tmpfs 优先**：默认尝试使用 **Tmpfs**（内存文件系统）作为存储后端，读写速度极快且重启即焚，具备极高的隐蔽性。
* **自动镜像回退**：自动检测环境是否支持 XATTR（SELinux 必需）。如果 Tmpfs 不支持，则自动创建并挂载一个 2GB 的 `ext4` 循环镜像 (`modules.img`)，并具备自动修复损坏镜像的能力。

### 🐾 隐蔽模式 (Paw Pad / Nuke)
* **Sysfs 清理**：支持通过 `ioctl` 移除 KernelSU 在 Sysfs 中的挂载痕迹，提高 Root 环境的隐蔽性。

### 📱 现代化 WebUI
* 内置基于 Svelte + Vite 构建的管理面板。
* 支持深色/浅色主题切换、多语言支持（中/英/日/俄/西）。
* 实时查看存储使用率、挂载状态及系统日志。

---

## 🛠️ 架构原理

Meta-Hybrid Mount 的工作流程如下：

1.  **环境初始化**：初始化日志，伪装进程名为 `kworker`。
2.  **存储准备**：尝试挂载 Tmpfs，若失败或不支持扩展属性，则挂载/修复 `modules.img`。
3.  **库存扫描**：扫描模块目录，读取模块配置和模式（Auto/Magic）。
4.  **增量同步**：将变动的模块文件同步至运行时存储目录。
5.  **规划挂载**：
    * 生成 OverlayFS 层级结构（Lowerdirs）。
    * 识别需要 Magic Mount 的路径。
6.  **执行挂载**：按计划执行挂载操作。如果 Overlay 失败，该模块会自动加入 Magic Mount 队列重试。
7.  **状态保存**：保存运行时状态以供 WebUI 读取。

---

## ⚙️ 配置说明

配置文件位于 `/data/adb/meta-hybrid/config.toml`。您也可以通过 WebUI 进行可视化修改。

| 配置项 | 类型 | 默认值 | 说明 |
| :--- | :--- | :--- | :--- |
| `moduledir` | String | `/data/adb/modules/` | 模块源目录路径。 |
| `tempdir` | String | (自动选择) | 临时工作目录。留空则自动选择。 |
| `mountsource` | String | `KSU` | 挂载源名称，用于 OverlayFS 的 source 参数。 |
| `verbose` | Bool | `false` | 是否开启详细调试日志。 |
| `partitions` | Array | `[]` | 额外的挂载分区列表（除 system/vendor 等内置分区外）。 |
| `force_ext4` | Bool | `false` | 强制使用 `modules.img` 而不尝试 Tmpfs。 |
| `enable_nuke` | Bool | `false` | 启用 "肉垫" 模式（清理 Sysfs 痕迹）。 |
| `disable_umount` | Bool | `false` | 禁用 add_try_umount |

---

## 🖥️ WebUI 功能

安装模块后，您可以通过 KernelSU 的管理器访问 WebUI（或直接在浏览器中打开对应地址）。

* **状态 (Status)**：
    * 查看 `modules.img` 或 Tmpfs 的存储占用。
    * 查看内核版本、SELinux 状态、活跃挂载分区。
    * OverlayFS 与 Magic Mount 的模块统计。
* **配置 (Config)**：
    * 可视化编辑 `config.toml`。
    * 一键重载配置。
* **模块 (Modules)**：
    * 搜索和筛选已安装的模块。
    * **模式切换**：针对特定模块，强制指定使用 "OverlayFS" 或 "Magic Mount" 模式（解决特定模块导致的死循环问题）。
* **日志 (Logs)**：
    * 实时查看守护进程运行日志 (`daemon.log`)。
    * 支持日志等级筛选和搜索。

---

## 🔨 构建指南

本项目使用 Rust 的 `xtask` 模式进行构建，并集成了 WebUI 的构建流程。

### 环境要求
* **Rust**: Nightly 工具链 (建议使用 `rustup`)
* **Android NDK**: 版本 r27+
* **Node.js**: v20+ (用于构建 WebUI)
* **Java**: JDK 17 (用于环境配置)

### 构建命令

1.  **克隆仓库**
    ```bash
    git clone --recursive [https://github.com/YuzakiKokuban/meta-hybrid_mount.git](https://github.com/YuzakiKokuban/meta-hybrid_mount.git)
    cd meta-hybrid_mount
    ```

2.  **执行构建**
    使用 `xtask` 自动处理 WebUI 编译、Rust 交叉编译及 Zip 打包：
    ```bash
    # 构建 Release 版本 (包含 WebUI 和所有架构的二进制文件)
    cargo run -p xtask -- build --release
    ```

    构建产物将位于 `output/` 目录下。

3.  **仅构建二进制文件 (跳过 WebUI)**
    如果您只修改了 Rust 代码，可以跳过 WebUI 构建以节省时间：
    ```bash
    cargo run -p xtask -- build --release --skip-webui
    ```

### 支持架构
构建脚本默认编译以下架构：
* `aarch64-linux-android` (arm64)
* `x86_64-linux-android` (x64)
* `riscv64-linux-android` (riscv64)

---

## 🤝 贡献与致谢

* 感谢所有开源社区的贡献者。
* 我们的姊妹项目[Hymo](https://github.com/Anatdx/hymo)，欢迎支持，呱唧呱唧呱唧。
* 本项目使用了 `rustix`, `clap`, `serde`, `svelte` 等优秀的开源库。

## 📄 许可证

本项目遵循 **GNU General Public License v3.0 (GPL-3.0)** 开源协议。详情请参阅 [LICENSE](LICENSE) 文件。

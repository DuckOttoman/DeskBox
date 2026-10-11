# DeskBox

[English](README.en.md) | 简体中文

面向 Windows 10/11 x64 的桌面整理工具，使用 Rust、Tauri 2 和 WebView2。通过文件夹框整理桌面文件，并提供可调整大小的小组件和动态背景。

这是基于现有 DeskBox 可执行程序的可编译重建工程：前端资源和分类规则从原程序提取，Rust 后端按调用协议重新实现，随后加入交互修复和新功能。它不是逐字恢复的原始 Rust 源码。

## 功能

- **桌面分类**：按扩展名、快捷方式目标、关键词及学习记录分类；支持拖动、搜索、重命名、打开位置、属性和回收站操作。
- **文件夹框**：独立样式、尺寸、卷起/隐藏、图标名称开关，以及 GIF/视频背景和圆角裁切。
- **外框组合与内容合并**：拖到另一框上，停留不足 5 秒松手组合外框；达到 5 秒后松手合并内容。松手前只预览，拖离或取消不落实合并。
- **组合操作**：单击标题切换成员，拖动标题移动整组，原地左键长按 2 秒分离当前成员；各成员保留独立属性。内容合并提供还原点。
- **视频模式**：节能模式最高 854×480、24–30fps；原画模式使用原视频分辨率和帧率。隐藏、折叠及相关系统状态下暂停动态背景。
- **小组件**：时间与天气、系统状态、倒计时/番茄钟、便签/待办、相册；拖边框调整大小，自动适配小、中、大布局。
- **天气**：新建时通过 IP 获取近似城市，支持手动选择；中、大布局显示天气 Emoji。
- **相册**：通过菜单添加图片，默认存储在程序目录的 `Albums`，可更换存储目录。
- **系统状态**：CPU、内存及支持的 NVIDIA 指标；温度使用 Windows ACPI 热区计数器，固件读数不一定是 CPU 核心温度。
- **内存优化**：清理 DeskBox 及其 WebView2 的可回收工作集，显示结果，不降低视频画质。
- **布局备份**：小球右键提供备份和还原管理，每项显示名称、备份时间和数量，支持重命名与删除；还原前自动保存当前布局。

布局备份记录框、小组件的位置、尺寸、属性及外框组合关系，不保存桌面文件清单。还原后按当前桌面重新分类，保留当前便签、待办、相册内容及计时运行状态；已删除的小组件按空内容恢复。备份保存在用户数据目录的 `layout-backups.json`。

## 安装与更新

使用构建产出的联网安装程序，安装范围为当前用户。需要联网下载缺少的 WebView2 与 FFmpeg，下载及组件安装在后台隐藏执行。WebView2 已安装时复用现有运行时。

已有安装也可使用替换更新 ZIP：

1. 从悬浮球或托盘菜单退出 DeskBox。
2. 将 ZIP 内的 `deskbox-reconstructed.exe` 和 `WebView2Loader.dll` 覆盖到现有安装目录。
3. 保留 `media-tools`、`Albums` 及用户数据目录，重新启动。

默认安装目录为 `%LOCALAPPDATA%\Programs\DeskBox Reconstructed`。用户配置使用独立应用标识 `com.deskbox.reconstructed`。退出软件会恢复由本程序隐藏的桌面图标。

安装包、更新包和构建缓存不纳入源码仓库；在本地构建后生成。

## 从源码编译

需要 Windows x64、Rust 工具链与链接器：MSVC 配 Visual Studio C++ Build Tools，或 GNU 配 MinGW-w64。当前工程已在 Rust 1.99.0 GNU 环境完成 Release 编译与验证。运行 GUI 需要 WebView2 Runtime。

仓库包含可直接编译的 `frontend/`，无需 Node.js 或 npm。

```powershell
.\build.ps1
```

也可直接运行：

```powershell
cargo test --no-default-features --locked
cargo build --release --locked
```

程序输出到 `target/release/deskbox-reconstructed.exe`。`build.ps1` 同时复制到 `bin/`，并复制可用的 WebView2 DLL 和媒体工具。

修改 `ui/` 扩展后，先重新生成前端：

```powershell
python tools/patch_frontend.py
```

### 视频工具与安装包

Python 3 环境中准备 FFmpeg 后，生成联网安装程序：

```powershell
python -m pip install imageio-ffmpeg
python tools/prepare_media_tools.py
python tools/build_installer.py --online
```

安装脚本会准备 NSIS；构建阶段需要联网。生成文件位于 `deliverables/`。便携运行时须保留 `media-tools/` 及其中许可证和版本说明。Rust 编译本身不要求 FFmpeg。

特殊的本机 TLS 环境可使用 `.\build.ps1 -LocalToolchain`，相关工具保存在 `.tools/`，不修改全局 Rust 安装。普通环境优先使用标准构建方式。

## 项目结构

| 目录 | 内容 |
|---|---|
| `src/` | Rust 后端、分类、布局、持久化及 Windows 适配 |
| `dist/` | 从原版提取的前端资源 |
| `frontend/` | 实际嵌入程序的前端 |
| `ui/` | 可读的菜单、合并动画、媒体与小组件扩展 |
| `tests/` | 核心回归及隔离 WebView 诊断驱动 |
| `installer/` | NSIS 安装与组件下载脚本 |
| `tools/` | 构建、前端生成及打包工具 |
| `docs/` | 使用说明、设计与实现记录 |
| `evidence/` | 经过筛选的重建依据与验证记录 |

## 验证与限制

核心测试、Release 编译、隔离 WebView 交互、布局备份重启持久化，以及安装/升级/卸载已验证。诊断数据与真实桌面隔离。模拟指针测试不能代替所有真实鼠标、多屏不同 DPI 和热插拔验证。

温度和 GPU 指标取决于硬件、固件及驱动，不支持时显示未提供。原画视频的解码能力取决于 WebView2 和系统编解码器；多个高分辨率视频背景会增加资源占用。旧背景缓存目前不会在每次替换后自动删除。

更多说明见 [小组件使用说明](docs/widgets.md) 和 [布局备份设计](docs/superpowers/specs/2026-10-11-layout-backup-design.md)。

## 许可证

保留仓库原有的 [GNU GPL v3 许可证](LICENSE)。第三方依赖、提取的资源与 FFmpeg 的许可应按其各自来源及随附说明处理。

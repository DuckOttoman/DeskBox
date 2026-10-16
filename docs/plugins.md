# DeskBox 插件开发（API v1）

插件是根目录含 manifest.json 的 ZIP，扩展名为 .deskbox-plugin。第一版支持无脚本的 theme 和 HTML/CSS/JS widget。最低宿主版本为 0.1.0。

## 构建与安装

在源码根目录运行：

```powershell
python tools/package_plugin.py examples/plugins/clock deliverables/plugins/clock-1.0.0.deskbox-plugin
python tools/package_plugin.py examples/plugins/mist-theme deliverables/plugins/mist-1.0.1.deskbox-plugin
```

设置 → 插件 → 本地导入，选择输出文件；检查名称、作者、来源及权限后确认安装。也可在 URL 安装中粘贴受信 HTTPS 包地址。安装预览不运行插件。启用后，小组件出现在“添加小组件”；主题一次启用一个。停用小组件会保留布局占位，卸载可选择是否删除独立设置。作者及来源是发布者声明，不代表身份已验证。

打包器只依赖 Python 标准库，按文件名排序并固定 ZIP 时间、权限，输出 SHA256；同一内容在相同 Python/zlib 环境中产生相同包。开发目录 .git、node_modules、__pycache__、.venv、venv、.env 及 .env.* 不打包。不要把密钥或个人数据放入插件资源。限制：ZIP 20 MiB，解压总计 50 MiB，512 文件，单文件 10 MiB。拒绝链接、绝对路径、上级路径、大小写碰撞、Windows 保留名与可执行文件。

## Manifest

完整 widget 示例见 ../../examples/plugins/clock/manifest.json。必填：apiVersion=1、id、name、version、author、type、minDeskBoxVersion；widget 还需 entry。id 使用稳定的小写反向域名（如 org.example.clock）；版本为三段数字。theme 固定读取 theme.json。description、homepage、updateUrl 可选。权限示例：

```json
{"permissions":{"network":["https://api.example.com"],"systemMetrics":true}}
```

network 只允许 HTTPS origin（无路径/查询）；systemMetrics 未声明时不可读取指标。插件设置天然按 ID 隔离。仅声明实际需要的权限；更新会重新显示权限变化。updateUrl 返回 {"id":"org.example.clock","version":"1.1.0","url":"https://example.com/clock-1.1.0.deskbox-plugin","sha256":"64位十六进制"}。仅手动检查更新。

## 小组件 SDK

宿主在 entry 脚本前注入 window.DeskBox，无需引用外部 SDK。所有请求方法返回 Promise；事件订阅返回取消订阅函数。

| 方法 | 约定 |
| --- | --- |
| ready() | 返回 {theme,size:{width,height,form},visible} |
| getTheme() | 获取当前主题 |
| onThemeChange(callback) | 主题变更 |
| onResize(callback) | {width,height,form}，尺寸为逻辑像素，form 为 small/medium/large |
| onVisibilityChange(callback) | 可见性布尔值；隐藏时暂停计时/动画 |
| getSettings() | 读取当前插件的 JSON 设置 |
| setSettings(value) | 保存 JSON，单插件最多 256 KiB |
| getSystemMetrics() | 读取宿主已有采样；需要 systemMetrics 权限 |
| fetch(url) | GET 请求，返回 {status,body,contentType}；body 为文本 |

clock 示例展示初始化、尺寸、可见性及持久化 12/24 小时制。销毁 iframe 后宿主清理实例；不要假定停用后脚本仍运行。theme 为宿主提供的主题对象，应容忍新增字段。系统指标沿用宿主采样结构，不保证传感器始终可用，应处理缺失值。

iframe 只允许脚本；没有同源、弹窗、顶层导航、文件系统、Shell 或任意 Tauri invoke 能力。不要直接使用 fetch、外部脚本或 CDN。SDK fetch 由宿主代理，检查已授权 origin，逐跳验证 HTTPS 与公网地址，拒绝本机、私网、链路本地及元数据目标，并限制超时、响应大小和并发。请求失败应显示组件内错误，不能通过自行联网绕过权限。

## 主题

Mist 示例只包含 JSON，不执行 JS，也不向宿主页注入 CSS。theme.json 使用 variables 映射：--db-plugin-background、--db-plugin-color、--db-plugin-accent、--db-plugin-border、--db-plugin-radius、--db-plugin-shadow、--db-plugin-font。值为受限制 CSS 字符串，禁止分号、花括号、HTML、url() 和 expression。宿主仅应用固定变量，不开放任意选择器或布局覆盖。

## 静态目录与发布

目录结构为 {"schemaVersion":1,"entries":[...]}。每项含 id、name、description、author、type、version、url、sha256，见 ../../server/market/catalog.json。通过独立配置的 HTTPS 目录提供发现功能；始终可直接安装 URL 或本地包。将包放在不可变的版本路径，发布前重新核对 SHA256。目录中的 IP URL 是本项目部署目标，只有服务成功部署且证书受信后才能在线安装；不要关闭证书校验。

发布前：打包并检查 ZIP 内容；本地导入预览；安装/启用；添加及缩放；切换主题；隐藏/恢复；保存设置后重启；停用/启用；卸载并验证保留或删除设置。校验命令：python -m unittest discover -s tests -p test_plugin_packaging.py。

主题 JSON 可另设顶层 `"backgroundImage":"assets/mist.png"`。仅接受包内已存在的 PNG/JPG/JPEG/WebP/GIF 路径；拒绝绝对路径、上级路径、SVG 和远程 URL。宿主将校验后的资源转换为栅格图片 data URL，不开放任意 CSS URL。变量值还禁止反斜杠与 @，避免 CSS 转义和导入。完整示例见 Mist 1.0.1。

CSS 值在忽略空白后禁止 url()、image-set()、image()、src() 与 expression。包内栅格资源仅通过 backgroundImage 引用，CSS 值不能授权联网加载。

资源使用普通 `<script src="main.js">`、样式表及包内图片。第一版不解析 JavaScript 模块依赖；使用构建工具将依赖打包为普通脚本，再制作插件包。CDN 脚本和直接联网会被阻止，请使用 SDK 的授权 fetch。

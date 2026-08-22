# Codex Tray

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · 简体中文 · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

用于监控 Codex 剩余额度的原生 Windows 系统托盘指示器。

## 概述

Codex Tray 无需让 Codex 应用或 CLI 始终位于前台，即可随时查看当前额度。它作为一个轻量的 Windows 系统托盘应用运行，复用当前用户已认证的 Codex CLI 会话，并在鼠标悬停于图标时显示紧凑的额度面板。

应用仅与本地安装的 `codex app-server` 通信。它不会要求提供 API 密钥，也不会直接读取或复制 `~/.codex/auth.json`。

## 功能

- 通过 `account/rateLimits/updated` 服务器通知实时更新额度。
- 支持 DPI 的紧凑面板，采用稳定的 `标签: 值` 行布局。
- 支持 Windows 明暗主题、强调色和透明效果。
- 针对额度等级和错误状态提供像素对齐的托盘图标。
- 鼠标悬停时显示面板，移开后自动隐藏。
- 右键菜单提供按需刷新、打开可执行文件目录、随 Windows 启动开关和明确的关闭操作。
- 托盘图标不显示系统工具提示。
- 分别显示加载、重连、需要认证、订阅不可用、CLI 缺失、额度耗尽和 app-server 错误状态。

## 系统要求

- Windows 11 x86-64。
- `PATH` 中可用的 [Codex CLI](https://learn.chatgpt.com/docs/codex/cli)。
- 使用 `codex login` 创建的已认证 Codex CLI 会话。

Codex Tray 目前仅实现了原生 Windows 后端。在 Linux、macOS 和 Windows ARM64 的平台后端完成实现和测试之前，不会发布这些平台的构建产物。

## 安装

1. 打开[最新 GitHub Release](https://github.com/psimonov/codex-tray/releases/latest)。
2. 下载 `codex-tray-<version>-windows-x86_64.exe` 及其 `.sha256` 文件。
3. 验证 SHA-256 校验和。
4. 将可执行文件移动到任意固定目录并运行。

在 PowerShell 中验证校验和的示例：

```powershell
Get-FileHash .\codex-tray-0.3.0-windows-x86_64.exe -Algorithm SHA256
```

无需安装程序。Release 仅包含一个便携式可执行文件；`codex` 命令仍是外部运行时依赖。

## 快速开始

```powershell
codex login
.\codex-tray-0.3.0-windows-x86_64.exe
```

应用会以隐藏状态启动，并在 Windows 系统托盘中添加图标。

## 使用方法

- 将鼠标悬停在托盘图标上以显示额度面板。
- 将鼠标移开以隐藏面板。
- 右键单击图标可隐藏面板并打开右键菜单。
- 选择**立即刷新**，可通过现有 app-server 连接立即重复 `account/read` 和 `account/rateLimits/read`。
- 选择**打开应用目录**，可打开当前运行的可执行文件所在目录。
- 切换**随 Windows 启动**，可在当前用户的 `Run` 注册表项中添加或移除当前可执行文件的路径。
- 选择**关闭**以停止 Codex Tray 及其 app-server 子进程。

额度更新通过与 `codex app-server` 的单个持久连接到达。Codex Tray 会在连接建立时读取一次账户和额度信息，仅在用户明确刷新时重复这两个请求，合并后续不完整的更新通知，并在 app-server 意外退出后重新连接。

## 配置

Codex Tray 不使用配置文件或环境变量。可选的 Windows 自启动功能通过右键菜单控制，并始终使用运行中可执行文件的动态检测路径。

## 从源代码构建

仓库固定了项目所需的 Rust 工具链。

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

生成的可执行文件位于 `target\release\codex-tray.exe`。

## 测试

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## 发布

版本标签采用 `vMAJOR.MINOR.PATCH` 格式。GitHub Actions 会验证标签与 `Cargo.toml` 中的版本一致，执行项目检查，构建 Windows x86-64 可执行文件，并在同一个 GitHub Release 中发布可执行文件及其 SHA-256 校验和。

项目目前仅支持 Windows，因此只发布 Windows x86-64 构建产物。这是一项明确的平台决策，而不是未经验证的跨平台支持声明。

## 安全

请参阅 [SECURITY.md](SECURITY.md) 了解受支持版本和私密漏洞报告方式。请勿在公开 issue 中披露漏洞。

## 参与贡献

请参阅 [CONTRIBUTING.md](CONTRIBUTING.md) 了解开发流程和 commit 要求。

## 许可证

Codex Tray 采用 [MIT 许可证](LICENSE)。

## 协议参考

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)

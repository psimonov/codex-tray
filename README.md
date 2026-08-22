# Codex Tray

English · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

A native Windows tray indicator for monitoring the remaining Codex usage quota.

## Overview

Codex Tray keeps the current Codex quota visible without requiring the Codex app or CLI to stay in the foreground. It runs as a small Windows system-tray application, reuses the current user's authenticated Codex CLI session, and displays a compact quota panel on hover.

The application communicates only with the locally installed `codex app-server`. It does not request an API key and does not read or copy `~/.codex/auth.json` directly.

## Features

- Live quota updates through `account/rateLimits/updated` server notifications.
- Compact DPI-aware panel with stable `Label: value` rows.
- Windows light/dark theme, accent color, and transparency support.
- Pixel-aligned tray icons for quota levels and error states.
- Hover to show the panel; move away to hide it.
- Embedded translations for 12 languages, with the system language selected by default.
- Context menu with on-demand refresh, executable-folder access, Windows startup control, and an explicit close action.
- Portable language and startup settings stored next to the executable.
- No system tooltip over the tray icon.
- Distinct loading, reconnecting, authentication, subscription, missing CLI, exhausted quota, and app-server error states.

## Requirements

- Windows 11 x86-64.
- [Codex CLI](https://learn.chatgpt.com/docs/codex/cli) available in `PATH`.
- An authenticated Codex CLI session created with `codex login`.

Codex Tray currently implements only a native Windows backend. Linux, macOS, and Windows ARM64 artifacts are intentionally not published until those targets have implemented and tested platform backends.

## Installation

1. Open the [latest GitHub Release](https://github.com/psimonov/codex-tray/releases/latest).
2. Download `codex-tray-<version>-windows-x86_64.exe` and its `.sha256` file.
3. Verify the SHA-256 checksum.
4. Move the executable to any permanent writable folder and run it.

Example checksum verification in PowerShell:

```powershell
Get-FileHash .\codex-tray-0.4.0-windows-x86_64.exe -Algorithm SHA256
```

No installer is required. The release is a single portable executable; the `codex` command remains an external runtime requirement.

## Quick start

```powershell
codex login
.\codex-tray-0.4.0-windows-x86_64.exe
```

The application starts hidden and adds its icon to the Windows system tray.

## Usage

- Hover over the tray icon to show the quota panel.
- Move the pointer away from the icon to hide the panel.
- Right-click the icon to hide the panel and open the context menu.
- Open **Language** and choose **System language** or a specific language. Changes apply immediately.
- Select **Refresh now** to immediately repeat `account/read` and `account/rateLimits/read` over the existing app-server connection.
- Select **Open application folder** to open the directory containing the running executable.
- Toggle **Start with Windows** to register or remove the current executable path under the current user's Windows `Run` key.
- Select **Close** to stop Codex Tray and its app-server child process.

Quota updates arrive over one persistent `codex app-server` connection. Codex Tray performs an initial account and rate-limit read, repeats both reads only after an explicit refresh, merges subsequent sparse update notifications, and reconnects after an unexpected app-server exit.

## Configuration

On first launch, Codex Tray creates `codex-tray.json` next to the executable. The file stores the selected language and Windows startup preference:

```json
{
  "language": "system",
  "start_with_windows": false
}
```

`language` accepts `system`, `en`, `es`, `fr`, `pt`, `de`, `it`, `ru`, `zh-CN`, `hi`, `ar`, `ja`, or `ko`. The configuration file is the source of truth. The user's Windows `Run` entry is synchronized from `start_with_windows` and always uses the dynamically detected path of the running executable. An existing startup entry is imported when the configuration file is first created.

## Build from source

The repository pins the Rust toolchain required by the project.

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

The resulting executable is `target\release\codex-tray.exe`.

## Testing

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Releases

Version tags use the `vMAJOR.MINOR.PATCH` format. GitHub Actions validates the tag against `Cargo.toml`, runs the project checks, builds the Windows x86-64 executable, and publishes both the executable and its SHA-256 checksum in one GitHub Release.

The project is Windows-only today, so only Windows x86-64 artifacts are published. This is an explicit platform decision rather than an unverified claim of cross-platform support.

## Security

See [SECURITY.md](SECURITY.md) for supported versions and private vulnerability reporting instructions. Do not disclose security vulnerabilities in public issues.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the development workflow and commit requirements.

## License

Codex Tray is available under the [MIT License](LICENSE).

## Protocol reference

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)

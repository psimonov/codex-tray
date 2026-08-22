# Agent Development Guide

This file is the project-level source of truth for AI-assisted development in Codex Tray. It applies to the entire repository. User instructions and organization-level regulations take precedence when they are more specific.

## Product contract

- Codex Tray is a native Windows 11 system-tray application written in Rust.
- The application starts without showing a normal window. Hovering over the tray icon shows the quota panel; moving the pointer away hides it.
- Right-clicking the tray icon hides the panel and opens a context menu with on-demand refresh, the executable folder, the Windows startup toggle, and **Close**.
- The tray icon has no system tooltip.
- The Windows startup entry is stored under the current user's `Run` key. Always derive its command from the running executable; never hardcode an installation path.
- The displayed percentage is the remaining Codex quota. The bar and tray glyph drain from top to bottom as the quota decreases.
- Panel rows use a stable `Label: value` layout and must not move or resize during updates.
- Respect Windows DPI, light/dark mode, accent color, and transparency settings. Preserve warning colors for critically low quota.

## Codex integration

- Keep one persistent `codex app-server --stdio` child process.
- Perform the initial `account/read` and `account/rateLimits/read` requests once per connection, and repeat them only after an explicit user refresh.
- Receive subsequent quota changes from `account/rateLimits/updated`; do not reintroduce periodic polling.
- Merge sparse update notifications without discarding account metadata or unchanged rate-limit fields.
- Reconnect after an unexpected app-server exit and terminate the child process when Codex Tray closes.
- Use the authenticated session managed by Codex CLI. Never request an API key or directly read, copy, log, or modify `~/.codex/auth.json`.
- Preserve distinct UI states for loading, reconnecting, ready, exhausted quota, authentication required, missing subscription/access, missing `codex` executable, and general app-server failure.

## Architecture

- `src/codex.rs` owns app-server transport, protocol parsing, reconnection, and platform-independent usage data.
- `src/platform.rs` is the platform boundary.
- `src/ui.rs` owns the Windows tray, hover panel, menu, autostart registration, theming, DPI behavior, and status presentation.
- `src/main.rs` wires the worker and platform UI together.
- `build.rs` generates and embeds application and tray icon resources. Inspect icon changes at 16, 20, 24, and 32 pixels at minimum.
- Keep platform-independent logic out of the Windows UI module where practical. Do not claim support for an OS or architecture that is not built and tested in CI.

## Development rules

- Use stable Rust pinned by `rust-toolchain.toml` and locked dependencies from `Cargo.lock`.
- Verify dependency and toolchain updates against current official documentation before adopting them.
- Keep source code formatted with `rustfmt` and warning-free under Clippy.
- Add or update deterministic tests for protocol parsing, state merging, status classification, path quoting, and other behavior that does not require a live account.
- Preserve CRLF-independent behavior and save every text file with LF line endings. `.gitattributes` enforces the repository policy.
- Do not add telemetry, network calls outside the local Codex app-server integration, or persistent storage without an explicit product decision.
- Keep the executable portable: application icons and required resources must remain embedded in the single release EXE. Codex CLI remains an external runtime requirement.

## Required verification

Run these commands before committing:

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --release --locked
```

For UI or icon changes, also run the built executable on Windows 11 and inspect tray behavior at relevant DPI and theme settings.

## Documentation

- `README.md` is canonical and written in English.
- Keep all 12 README variants synchronized in the language order defined by the language switcher: English, Español, Français, Português, Deutsch, Italiano, Русский, 简体中文, हिन्दी, العربية, 日本語, and 한국어.
- Do not translate commands, API method names, file names, paths, artifact names, or code examples.
- Update `SECURITY.md`, `CONTRIBUTING.md`, and this file whenever their contracts change.
- Internal repository links should be relative. Public claims about platform support must match CI and published release assets.

## Git and releases

- Use Conventional Commits with an English imperative description and no trailing period.
- Keep each commit focused on one logical change. Mark breaking changes with `!` and a `BREAKING CHANGE:` footer.
- Follow Semantic Versioning. Never move or reuse a published version tag.
- Release tags use `vMAJOR.MINOR.PATCH` and must match `Cargo.toml`.
- Releases are built by `.github/workflows/release.yml`. The current supported release target is Windows x86-64, with an EXE and a SHA-256 checksum.
- Do not publish Linux, macOS, or Windows ARM64 artifacts until their platform backends and CI coverage exist.

## Completion checklist

Before handing off a change, confirm that:

1. The requested behavior is implemented without breaking the product contract.
2. Tests cover the changed deterministic behavior.
3. The required verification commands pass.
4. User-facing facts are synchronized across all README languages.
5. All tracked text files use LF.
6. The commit and release metadata follow the repository conventions.

# Contributing to Codex Tray

Thank you for helping improve Codex Tray.

## Development environment

- Windows 11 x86-64
- Rust toolchain pinned by `rust-toolchain.toml`
- Codex CLI available in `PATH` for runtime integration checks

Clone the repository and build with locked dependencies:

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --locked
```

## Required checks

Run all checks before opening a pull request:

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build --release --locked
```

Changes to tray icons must be inspected at every embedded size: 16, 20, 24, and 32 pixels.

## Pull requests

- Keep each pull request focused on one logical change.
- Update both `README.md` and `README.ru.md` when user-facing facts change.
- Document platform support accurately; do not claim untested targets.
- Preserve the channel boundary between shared Codex logic and the Windows UI backend.
- Add or update tests for behavior that can be verified without a live account.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/) with an English imperative description and no trailing period, for example:

```text
fix: keep the quota panel hidden while the tray menu is open
```

Use `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, or `revert` as appropriate. Mark breaking changes with `!` and a `BREAKING CHANGE:` footer.

## Security

Do not open a public issue for a suspected vulnerability. Follow [SECURITY.md](SECURITY.md).

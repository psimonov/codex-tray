//! Platform UI boundary.
//!
//! Shared Codex querying and data models live outside this module. Future
//! macOS/Linux tray backends can implement the same channel-based boundary
//! without changing the worker or quota semantics.

#[cfg(windows)]
#[path = "ui.rs"]
mod windows;

#[cfg(windows)]
pub use windows::{run, startup_error_message};

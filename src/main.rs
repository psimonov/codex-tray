#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod codex;
mod platform;

#[cfg(windows)]
use std::{sync::mpsc, thread};

#[cfg(windows)]
use windows::{
    Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW},
    core::PCWSTR,
};

#[cfg(windows)]
fn main() {
    let (update_tx, update_rx) = mpsc::channel();
    let (command_tx, command_rx) = mpsc::channel();
    thread::spawn(move || codex::run_worker(update_tx, command_rx));

    if let Err(error) = platform::run(update_rx, command_tx) {
        show_error(&format!("Codex Tray не удалось запустить:\n\n{error}"));
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Codex Tray: backend for this operating system is not implemented yet");
}

#[cfg(windows)]
fn show_error(message: &str) {
    let message: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    unsafe {
        let _ = MessageBoxW(
            None,
            PCWSTR(message.as_ptr()),
            windows::core::w!("Codex Tray"),
            MB_OK | MB_ICONERROR,
        );
    }
}

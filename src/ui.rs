use std::{
    cell::RefCell,
    ffi::c_void,
    mem::size_of,
    path::Path,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    time::{Duration, Instant},
};

use chrono::{DateTime, Local};
use windows::{
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
        Graphics::{
            Dwm::{DWMWA_WINDOW_CORNER_PREFERENCE, DwmGetColorizationColor, DwmSetWindowAttribute},
            Gdi::{
                BeginPaint, CreateFontIndirectW, CreateSolidBrush, DeleteObject, EndPaint,
                FW_SEMIBOLD, FillRect, GetMonitorInfoW, GetStockObject, GetTextExtentPoint32W,
                HBRUSH, HFONT, HGDIOBJ, IntersectClipRect, InvalidateRect,
                MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow, PAINTSTRUCT, RestoreDC,
                SYSTEM_FONT, SaveDC, SelectObject, SetBkMode, SetPixelV, SetTextColor, TRANSPARENT,
                TextOutW,
            },
        },
        System::LibraryLoader::GetModuleHandleW,
        System::Registry::{
            HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_DWORD, RRF_RT_REG_SZ, RegDeleteKeyValueW,
            RegGetValueW, RegSetKeyValueW,
        },
        UI::{
            HiDpi::{
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow,
                GetSystemMetricsForDpi, SetProcessDpiAwarenessContext, SystemParametersInfoForDpi,
            },
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
                NOTIFYICONIDENTIFIER, Shell_NotifyIconGetRect, Shell_NotifyIconW,
            },
            WindowsAndMessaging::{
                AppendMenuW, CREATESTRUCTW, CW_USEDEFAULT, CreatePopupMenu, CreateWindowExW,
                DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW,
                GWLP_USERDATA, GetClientRect, GetCursorPos, GetMessageW, HICON, IDC_ARROW,
                IDI_APPLICATION, IMAGE_ICON, LR_DEFAULTCOLOR, LWA_ALPHA, LoadCursorW, LoadIconW,
                LoadImageW, MB_ICONERROR, MB_OK, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED,
                MSG, MessageBoxW, NONCLIENTMETRICSW, PostQuitMessage, RegisterClassExW,
                SM_CXSMICON, SPI_GETNONCLIENTMETRICS, SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
                SetForegroundWindow, SetLayeredWindowAttributes, SetTimer, SetWindowLongPtrW,
                SetWindowPos, ShowWindow, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD,
                TrackPopupMenu, WM_APP, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_MOUSEMOVE,
                WM_NCCREATE, WM_PAINT, WM_RBUTTONUP, WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED,
                WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
            },
        },
    },
    core::{BOOL, PCWSTR, Result as WinResult, w},
};

use crate::codex::{UsageSnapshot, WorkerCommand, WorkerUpdate};

const WINDOW_CLASS: PCWSTR = w!("CodexTrayStatusWindow");
const WINDOW_WIDTH: i32 = 320;
const WINDOW_HEIGHT: i32 = 195;
const TRAY_ID: u32 = 1;
const TRAY_MESSAGE: u32 = WM_APP + 1;
const TIMER_ID: usize = 1;
const MENU_AUTOSTART: usize = 1001;
const MENU_EXIT: usize = 1002;
const HOVER_HIDE_DELAY: Duration = Duration::from_millis(150);
const AUTOSTART_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const AUTOSTART_VALUE: PCWSTR = w!("Codex Tray");
const APP_ICON_RESOURCE: u16 = 1;
const STATUS_ICON_RESOURCES: &[(u32, u16)] = &[
    (0, 100),
    (5, 101),
    (25, 102),
    (50, 103),
    (75, 104),
    (95, 105),
    (100, 106),
];
const LOADING_ICON_RESOURCE: u16 = 107;
const ERROR_ICON_RESOURCE: u16 = 108;
const ACCOUNT_ICON_RESOURCE: u16 = 109;
const MISSING_ICON_RESOURCE: u16 = 110;

thread_local! {
    static APP: RefCell<Option<AppState>> = const { RefCell::new(None) };
}

struct AppState {
    updates: Receiver<WorkerUpdate>,
    commands: Sender<WorkerCommand>,
    snapshot: Option<UsageSnapshot>,
    last_error: Option<(String, i64)>,
    querying: bool,
    visible: bool,
    last_tray_hover: Option<Instant>,
    suppress_hover_until_leave: bool,
    tray_icon: Option<HICON>,
    tray_icon_resource: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiStatus {
    Loading,
    Refreshing,
    Ready,
    Exhausted,
    AccountRequired,
    SubscriptionRequired,
    CodexMissing,
    Error,
}

pub fn run(updates: Receiver<WorkerUpdate>, commands: Sender<WorkerCommand>) -> WinResult<()> {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = GetModuleHandleW(None)?;
        let cursor = LoadCursorW(None, IDC_ARROW)?;
        let icon = LoadIconW(
            Some(HINSTANCE(instance.0)),
            resource_name(APP_ICON_RESOURCE),
        )
        .or_else(|_| LoadIconW(None, IDI_APPLICATION))?;

        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            hInstance: HINSTANCE(instance.0),
            lpszClassName: WINDOW_CLASS,
            lpfnWndProc: Some(window_proc),
            hCursor: cursor,
            hIcon: icon,
            hIconSm: icon,
            hbrBackground: HBRUSH::default(),
            ..Default::default()
        };

        if RegisterClassExW(&class) == 0 {
            return Err(windows::core::Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_LAYERED,
            WINDOW_CLASS,
            w!("Codex limit"),
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let tray_icon_resource = LOADING_ICON_RESOURCE;
        let tray_icon = load_status_icon(hwnd, tray_icon_resource)?;
        APP.with(|app| {
            *app.borrow_mut() = Some(AppState {
                updates,
                commands,
                snapshot: None,
                last_error: None,
                querying: true,
                visible: false,
                last_tray_hover: None,
                suppress_hover_until_leave: false,
                tray_icon: Some(tray_icon),
                tray_icon_resource,
            });
        });

        add_tray_icon(hwnd, tray_icon)?;
        apply_rounded_corners(hwnd);
        apply_system_transparency(hwnd);
        position_near_tray(hwnd, false);
        SetTimer(Some(hwnd), TIMER_ID, 100, None);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_NCCREATE => {
                let create = lparam.0 as *const CREATESTRUCTW;
                if !create.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*create).lpCreateParams as isize);
                }
                LRESULT(1)
            }
            WM_CREATE => LRESULT(0),
            WM_TIMER if wparam.0 == TIMER_ID => {
                drain_updates(hwnd);
                update_hover_visibility(hwnd);
                LRESULT(0)
            }
            TRAY_MESSAGE => {
                let event = (lparam.0 as u32) & 0xffff;
                if event == WM_MOUSEMOVE {
                    show_hover_window(hwnd);
                } else if event == WM_RBUTTONUP {
                    hide_hover_window(hwnd);
                    show_context_menu(hwnd);
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                handle_menu_command(hwnd, wparam.0 & 0xffff);
                LRESULT(0)
            }
            WM_PAINT => {
                paint(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                let tray_icon = APP.with(|app| {
                    let mut app = app.borrow_mut();
                    if let Some(state) = app.as_mut() {
                        let _ = state.commands.send(WorkerCommand::Stop);
                        state.tray_icon.take()
                    } else {
                        None
                    }
                });
                remove_tray_icon(hwnd);
                if let Some(icon) = tray_icon {
                    let _ = DestroyIcon(icon);
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}

unsafe fn drain_updates(hwnd: HWND) {
    unsafe {
        let mut changed = false;
        APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else { return };
            loop {
                match state.updates.try_recv() {
                    Ok(WorkerUpdate::Querying) => {
                        state.querying = true;
                        changed = true;
                    }
                    Ok(WorkerUpdate::Snapshot(snapshot)) => {
                        state.snapshot = Some(snapshot);
                        state.last_error = None;
                        state.querying = false;
                        changed = true;
                    }
                    Ok(WorkerUpdate::Error { message, at }) => {
                        state.last_error = Some((message, at));
                        state.querying = false;
                        changed = true;
                    }
                    Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
                }
            }
        });

        if changed {
            update_tray_visual(hwnd);
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }
}

unsafe fn paint(hwnd: HWND) {
    unsafe {
        let mut paint = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut paint);
        let mut client = RECT::default();
        let _ = GetClientRect(hwnd, &mut client);

        let palette = system_palette();
        fill_rect(hdc, &client, palette.background);

        let (snapshot, error, querying) = APP.with(|app| {
            let app = app.borrow();
            let state = app.as_ref();
            (
                state.and_then(|value| value.snapshot.clone()),
                state.and_then(|value| value.last_error.clone()),
                state.is_some_and(|value| value.querying),
            )
        });
        let status = ui_status(snapshot.as_ref(), error.as_ref(), querying);

        if let Some(snapshot) = snapshot {
            draw_snapshot(hwnd, hdc, &snapshot, status, error.as_ref());
        } else {
            draw_loading(hwnd, hdc, status, error.as_ref());
        }

        let _ = EndPaint(hwnd, &paint);
    }
}

unsafe fn draw_snapshot(
    hwnd: HWND,
    hdc: windows::Win32::Graphics::Gdi::HDC,
    snapshot: &UsageSnapshot,
    status: UiStatus,
    _error: Option<&(String, i64)>,
) {
    unsafe {
        let percent_font = create_system_font(hwnd, true);
        let body_font = create_system_font(hwnd, false);
        SetBkMode(hdc, TRANSPARENT);

        let track = RECT {
            left: scale(hwnd, 10),
            top: scale(hwnd, 9),
            right: client_width(hwnd) - scale(hwnd, 10),
            bottom: scale(hwnd, 29),
        };
        let corner_radius = scale(hwnd, 3);
        let palette = system_palette();
        let panel_color = palette.background;
        let track_color = palette.track;
        fill_rounded_rect(hdc, &track, track_color, panel_color, corner_radius);

        let remaining_percent = 100_u32.saturating_sub(snapshot.used_percent);
        let fill_width = ((track.right - track.left) * remaining_percent as i32) / 100;
        let fill_right = track.left + fill_width;
        if fill_width > 0 {
            let fill = RECT {
                right: fill_right,
                ..track
            };
            let fill_radius = corner_radius.min(fill_width / 2);
            fill_rounded_rect(
                hdc,
                &fill,
                remaining_color(remaining_percent),
                track_color,
                fill_radius,
            );
        }

        select_font(hdc, percent_font);
        draw_contrast_percent(
            hdc,
            &track,
            fill_right,
            remaining_percent,
            remaining_color(remaining_percent),
            palette,
        );

        let rows = snapshot_rows(snapshot, status);
        draw_rows(hwnd, hdc, percent_font, body_font, &rows, status);

        let _ = SelectObject(hdc, GetStockObject(SYSTEM_FONT));
        let _ = DeleteObject(HGDIOBJ(percent_font.0));
        let _ = DeleteObject(HGDIOBJ(body_font.0));
    }
}

unsafe fn draw_loading(
    hwnd: HWND,
    hdc: windows::Win32::Graphics::Gdi::HDC,
    status: UiStatus,
    error: Option<&(String, i64)>,
) {
    unsafe {
        let title_font = create_system_font(hwnd, true);
        let body_font = create_system_font(hwnd, false);
        SetBkMode(hdc, TRANSPARENT);
        let track = RECT {
            left: scale(hwnd, 10),
            top: scale(hwnd, 9),
            right: client_width(hwnd) - scale(hwnd, 10),
            bottom: scale(hwnd, 29),
        };
        fill_rounded_rect(
            hdc,
            &track,
            system_palette().track,
            system_palette().background,
            scale(hwnd, 3),
        );
        select_font(hdc, title_font);
        draw_centered_text(hdc, &track, status_title(status), status_color(status));

        let updated = error
            .map(|(_, at)| format_datetime(*at))
            .unwrap_or_else(|| "—".into());
        let rows = vec![
            ("Статус", status_title(status).to_owned()),
            ("Осталось", "—".into()),
            ("Использовано", "—".into()),
            ("Тариф", "—".into()),
            ("Окно", "—".into()),
            ("Сброс", "—".into()),
            ("Кредиты", "—".into()),
            ("Обновлено", updated),
        ];
        draw_rows(hwnd, hdc, title_font, body_font, &rows, status);
        let _ = SelectObject(hdc, GetStockObject(SYSTEM_FONT));
        let _ = DeleteObject(HGDIOBJ(title_font.0));
        let _ = DeleteObject(HGDIOBJ(body_font.0));
    }
}

fn snapshot_rows(snapshot: &UsageSnapshot, status: UiStatus) -> Vec<(&'static str, String)> {
    let remaining = 100_u32.saturating_sub(snapshot.used_percent);
    let reset = snapshot
        .resets_at
        .map(format_datetime)
        .unwrap_or_else(|| "неизвестно".into());
    vec![
        ("Статус", status_title(status).to_owned()),
        ("Осталось", format!("{remaining}%")),
        ("Использовано", format!("{}%", snapshot.used_percent)),
        (
            "Тариф",
            snapshot
                .plan_type
                .clone()
                .unwrap_or_else(|| "неизвестно".into()),
        ),
        ("Окно", format_window(snapshot.window_duration_mins)),
        ("Сброс", reset),
        ("Кредиты", credits_text(snapshot)),
        ("Обновлено", format_datetime(snapshot.updated_at)),
    ]
}

unsafe fn draw_rows(
    hwnd: HWND,
    hdc: windows::Win32::Graphics::Gdi::HDC,
    key_font: HFONT,
    value_font: HFONT,
    rows: &[(&str, String)],
    status: UiStatus,
) {
    unsafe {
        let palette = system_palette();
        let key_x = scale(hwnd, 10);
        let value_x = scale(hwnd, 112);
        let first_y = scale(hwnd, 40);
        let row_height = scale(hwnd, 18);
        for (index, (key, value)) in rows.iter().enumerate() {
            let y = first_y + index as i32 * row_height;
            select_font(hdc, key_font);
            SetTextColor(hdc, palette.key);
            text_out(hdc, key_x, y, &format!("{key}:"));
            select_font(hdc, value_font);
            SetTextColor(
                hdc,
                if index == 0 {
                    status_color(status)
                } else {
                    palette.value
                },
            );
            text_out(hdc, value_x, y, value);
        }
    }
}

unsafe fn draw_centered_text(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: &RECT,
    value: &str,
    color: COLORREF,
) {
    unsafe {
        let text = wide(value);
        let visible = &text[..text.len() - 1];
        let mut size = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, visible, &mut size);
        SetTextColor(hdc, color);
        let _ = TextOutW(
            hdc,
            rect.left + (rect.right - rect.left - size.cx) / 2,
            rect.top + (rect.bottom - rect.top - size.cy) / 2,
            visible,
        );
    }
}

unsafe fn draw_contrast_percent(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    track: &RECT,
    fill_right: i32,
    percent: u32,
    fill_color: COLORREF,
    palette: SystemPalette,
) {
    unsafe {
        let text = wide(&format!("{percent}%"));
        let visible_text = &text[..text.len() - 1];
        let mut size = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, visible_text, &mut size);
        let x = track.left + (track.right - track.left - size.cx) / 2;
        let y = track.top + (track.bottom - track.top - size.cy) / 2;

        if fill_right < track.right {
            let saved = SaveDC(hdc);
            IntersectClipRect(hdc, fill_right, track.top, track.right, track.bottom);
            SetTextColor(hdc, palette.value);
            let _ = TextOutW(hdc, x, y, visible_text);
            let _ = RestoreDC(hdc, saved);
        }

        if fill_right > track.left {
            let saved = SaveDC(hdc);
            IntersectClipRect(hdc, track.left, track.top, fill_right, track.bottom);
            SetTextColor(hdc, contrast_color(fill_color));
            let _ = TextOutW(hdc, x, y, visible_text);
            let _ = RestoreDC(hdc, saved);
        }
    }
}

unsafe fn add_tray_icon(hwnd: HWND, icon: HICON) -> WinResult<()> {
    unsafe {
        let mut data = tray_data(hwnd);
        data.uFlags = NIF_MESSAGE | NIF_ICON;
        data.uCallbackMessage = TRAY_MESSAGE;
        data.hIcon = icon;
        if !Shell_NotifyIconW(NIM_ADD, &data).as_bool() {
            return Err(windows::core::Error::from_thread());
        }
        Ok(())
    }
}

unsafe fn update_tray_visual(hwnd: HWND) {
    unsafe {
        let desired_resource = APP.with(|app| {
            app.borrow()
                .as_ref()
                .map(tray_resource_for_state)
                .unwrap_or(LOADING_ICON_RESOURCE)
        });

        let current_resource = APP.with(|app| {
            app.borrow()
                .as_ref()
                .map(|state| state.tray_icon_resource)
                .unwrap_or(desired_resource)
        });
        let replacement = if desired_resource != current_resource {
            load_status_icon(hwnd, desired_resource).ok()
        } else {
            None
        };

        let mut icon_for_update = None;
        let mut old_icon = None;
        APP.with(|app| {
            let mut app = app.borrow_mut();
            if let Some(state) = app.as_mut() {
                if let Some(icon) = replacement {
                    old_icon = state.tray_icon.replace(icon);
                    state.tray_icon_resource = desired_resource;
                }
                icon_for_update = state.tray_icon;
            }
        });

        let mut data = tray_data(hwnd);
        if let Some(icon) = icon_for_update {
            data.uFlags = NIF_ICON;
            data.hIcon = icon;
            let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
        }
        if let Some(icon) = old_icon {
            let _ = DestroyIcon(icon);
        }
    }
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    unsafe {
        let data = tray_data(hwnd);
        let _ = Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

fn tray_data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    }
}

unsafe fn show_hover_window(hwnd: HWND) {
    unsafe {
        let mut cursor = POINT::default();
        if GetCursorPos(&mut cursor).is_err()
            || !tray_icon_rect(hwnd).is_some_and(|rect| point_in_rect(cursor, rect))
        {
            return;
        }
        APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else { return };
            if state.suppress_hover_until_leave {
                return;
            }
            state.last_tray_hover = Some(Instant::now());
            if !state.visible {
                state.visible = true;
                position_near_tray(hwnd, true);
            }
        });
    }
}

unsafe fn update_hover_visibility(hwnd: HWND) {
    unsafe {
        let mut cursor = POINT::default();
        let cursor_over_icon = GetCursorPos(&mut cursor).is_ok()
            && tray_icon_rect(hwnd).is_some_and(|rect| point_in_rect(cursor, rect));

        APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else { return };
            if cursor_over_icon {
                if !state.suppress_hover_until_leave {
                    state.last_tray_hover = Some(Instant::now());
                }
                return;
            }
            state.suppress_hover_until_leave = false;
            if state.visible
                && state
                    .last_tray_hover
                    .is_some_and(|hover| hover.elapsed() >= HOVER_HIDE_DELAY)
            {
                state.visible = false;
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        });
    }
}

unsafe fn hide_hover_window(hwnd: HWND) {
    unsafe {
        APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else { return };
            state.visible = false;
            state.last_tray_hover = None;
            state.suppress_hover_until_leave = true;
            let _ = ShowWindow(hwnd, SW_HIDE);
        });
    }
}

unsafe fn tray_icon_rect(hwnd: HWND) -> Option<RECT> {
    unsafe {
        let identifier = NOTIFYICONIDENTIFIER {
            cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
            hWnd: hwnd,
            uID: TRAY_ID,
            ..Default::default()
        };
        Shell_NotifyIconGetRect(&identifier).ok()
    }
}

fn point_in_rect(point: POINT, rect: RECT) -> bool {
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
}

unsafe fn show_context_menu(hwnd: HWND) {
    unsafe {
        let menu = match CreatePopupMenu() {
            Ok(menu) => menu,
            Err(_) => return,
        };
        let autostart = wide("Запускать вместе с Windows");
        let autostart_state = if autostart_enabled() {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        let close = wide("Закрыть");
        let _ = AppendMenuW(
            menu,
            MF_STRING | autostart_state,
            MENU_AUTOSTART,
            PCWSTR(autostart.as_ptr()),
        );
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(menu, MF_STRING, MENU_EXIT, PCWSTR(close.as_ptr()));

        let mut cursor = POINT::default();
        let _ = GetCursorPos(&mut cursor);
        let _ = SetForegroundWindow(hwnd);
        let command = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD,
            cursor.x,
            cursor.y,
            None,
            hwnd,
            None,
        );
        let _ = DestroyMenu(menu);
        if command.0 != 0 {
            handle_menu_command(hwnd, command.0 as usize);
        }
    }
}

unsafe fn handle_menu_command(hwnd: HWND, command: usize) {
    unsafe {
        match command {
            MENU_AUTOSTART => {
                if let Err(error) = set_autostart(!autostart_enabled()) {
                    show_ui_error(hwnd, &error);
                }
            }
            MENU_EXIT => {
                let _ = DestroyWindow(hwnd);
            }
            _ => {}
        }
    }
}

fn autostart_enabled() -> bool {
    let mut size = 0_u32;
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            AUTOSTART_KEY,
            AUTOSTART_VALUE,
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut size),
        )
        .is_ok()
    }
}

fn set_autostart(enabled: bool) -> Result<(), String> {
    let result = unsafe {
        if enabled {
            let executable = std::env::current_exe()
                .map_err(|error| format!("Не удалось определить путь приложения: {error}"))?;
            let command = wide(&quote_executable(&executable));
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                AUTOSTART_KEY,
                AUTOSTART_VALUE,
                REG_SZ.0,
                Some(command.as_ptr().cast()),
                (command.len() * size_of::<u16>()) as u32,
            )
        } else {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, AUTOSTART_KEY, AUTOSTART_VALUE)
        }
    };

    if result.is_ok() {
        Ok(())
    } else {
        Err(format!(
            "Не удалось изменить автозапуск Windows (код {}).",
            result.0
        ))
    }
}

fn quote_executable(path: &Path) -> String {
    format!("\"{}\"", path.to_string_lossy())
}

unsafe fn show_ui_error(hwnd: HWND, message: &str) {
    unsafe {
        let message = wide(message);
        let _ = MessageBoxW(
            Some(hwnd),
            PCWSTR(message.as_ptr()),
            w!("Codex Tray"),
            MB_OK | MB_ICONERROR,
        );
    }
}

unsafe fn position_near_tray(hwnd: HWND, show: bool) {
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let work = if GetMonitorInfoW(monitor, &mut info).as_bool() {
            info.rcWork
        } else {
            RECT {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            }
        };

        let flags = if show {
            SWP_NOACTIVATE | SWP_SHOWWINDOW
        } else {
            SWP_NOACTIVATE
        };
        let height = scale(hwnd, WINDOW_HEIGHT);
        let margin = scale(hwnd, 16);
        let width = scale(hwnd, WINDOW_WIDTH);
        let _ = SetWindowPos(
            hwnd,
            Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
            work.right - width - margin,
            work.bottom - height - margin,
            width,
            height,
            flags,
        );
    }
}

unsafe fn apply_rounded_corners(hwnd: HWND) {
    unsafe {
        let preference: u32 = 2;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        );
    }
}

unsafe fn apply_system_transparency(hwnd: HWND) {
    unsafe {
        let transparency_enabled = read_personalization_dword("EnableTransparency") != Some(0);
        let alpha = if transparency_enabled { 238 } else { 255 };
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
    }
}

#[derive(Clone, Copy)]
struct SystemPalette {
    background: COLORREF,
    track: COLORREF,
    key: COLORREF,
    value: COLORREF,
}

fn system_palette() -> SystemPalette {
    let light = read_personalization_dword("AppsUseLightTheme").is_some_and(|value| value != 0);
    if light {
        SystemPalette {
            background: rgb(243, 243, 243),
            track: rgb(215, 215, 215),
            key: rgb(90, 90, 90),
            value: rgb(24, 24, 24),
        }
    } else {
        SystemPalette {
            background: rgb(32, 32, 32),
            track: rgb(58, 58, 58),
            key: rgb(175, 175, 175),
            value: rgb(245, 245, 245),
        }
    }
}

fn read_personalization_dword(value_name: &str) -> Option<u32> {
    let value_name = wide(value_name);
    let mut value = 0_u32;
    let mut size = size_of::<u32>() as u32;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            PCWSTR(value_name.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut c_void),
            Some(&mut size),
        )
    };
    result.is_ok().then_some(value)
}

fn contrast_color(background: COLORREF) -> COLORREF {
    let red = background.0 & 0xff;
    let green = (background.0 >> 8) & 0xff;
    let blue = (background.0 >> 16) & 0xff;
    if red * 299 + green * 587 + blue * 114 >= 150_000 {
        rgb(18, 18, 18)
    } else {
        rgb(250, 250, 250)
    }
}

unsafe fn client_width(hwnd: HWND) -> i32 {
    unsafe {
        let mut client = RECT::default();
        if GetClientRect(hwnd, &mut client).is_ok() {
            client.right - client.left
        } else {
            scale(hwnd, WINDOW_WIDTH)
        }
    }
}

fn status_title(status: UiStatus) -> &'static str {
    match status {
        UiStatus::Loading => "Загрузка",
        UiStatus::Refreshing => "Обновление",
        UiStatus::Ready => "Готово",
        UiStatus::Exhausted => "Лимит исчерпан",
        UiStatus::AccountRequired => "Требуется вход",
        UiStatus::SubscriptionRequired => "Нет активного доступа",
        UiStatus::CodexMissing => "Codex не найден",
        UiStatus::Error => "Ошибка Codex",
    }
}

fn status_color(status: UiStatus) -> COLORREF {
    match status {
        UiStatus::Error | UiStatus::Exhausted => rgb(255, 91, 110),
        UiStatus::AccountRequired | UiStatus::SubscriptionRequired | UiStatus::CodexMissing => {
            rgb(255, 181, 71)
        }
        UiStatus::Loading | UiStatus::Refreshing => rgb(101, 199, 242),
        UiStatus::Ready => rgb(52, 211, 153),
    }
}

fn ui_status(
    snapshot: Option<&UsageSnapshot>,
    error: Option<&(String, i64)>,
    querying: bool,
) -> UiStatus {
    if let Some((message, _)) = error {
        return classify_error(message);
    }
    if querying {
        return if snapshot.is_some() {
            UiStatus::Refreshing
        } else {
            UiStatus::Loading
        };
    }
    if snapshot.is_some_and(|value| value.used_percent >= 100 || value.limit_reached_type.is_some())
    {
        UiStatus::Exhausted
    } else if snapshot.is_some() {
        UiStatus::Ready
    } else {
        UiStatus::Loading
    }
}

fn classify_error(message: &str) -> UiStatus {
    let message = message.to_lowercase();
    if message.contains("не удалось запустить codex")
        || message.contains("codex: program not found")
    {
        UiStatus::CodexMissing
    } else if ["unauthorized", "not logged", "login", "auth", "401", "вход"]
        .iter()
        .any(|needle| message.contains(needle))
    {
        UiStatus::AccountRequired
    } else if [
        "subscription",
        "billing",
        "payment",
        "paid",
        "purchase",
        "403",
        "подпис",
        "оплат",
    ]
    .iter()
    .any(|needle| message.contains(needle))
    {
        UiStatus::SubscriptionRequired
    } else {
        UiStatus::Error
    }
}

fn tray_resource_for_state(state: &AppState) -> u16 {
    match ui_status(
        state.snapshot.as_ref(),
        state.last_error.as_ref(),
        state.querying,
    ) {
        UiStatus::Loading | UiStatus::Refreshing => LOADING_ICON_RESOURCE,
        UiStatus::AccountRequired | UiStatus::SubscriptionRequired => ACCOUNT_ICON_RESOURCE,
        UiStatus::CodexMissing => MISSING_ICON_RESOURCE,
        UiStatus::Error => ERROR_ICON_RESOURCE,
        UiStatus::Exhausted => status_icon_resource(0),
        UiStatus::Ready => state
            .snapshot
            .as_ref()
            .map(|snapshot| status_icon_resource(100_u32.saturating_sub(snapshot.used_percent)))
            .unwrap_or(LOADING_ICON_RESOURCE),
    }
}

fn credits_text(snapshot: &UsageSnapshot) -> String {
    if snapshot.unlimited_credits {
        "∞".into()
    } else {
        snapshot
            .credit_balance
            .clone()
            .unwrap_or_else(|| "0".into())
    }
}

fn format_window(minutes: Option<i64>) -> String {
    match minutes {
        Some(value) if value % 10_080 == 0 => format!("{} нед.", value / 10_080),
        Some(value) if value % 1_440 == 0 => format!("{} дн.", value / 1_440),
        Some(value) if value % 60 == 0 => format!("{} ч", value / 60),
        Some(value) => format!("{} мин", value),
        None => "окно неизвестно".into(),
    }
}

fn format_datetime(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .map(|date| date.with_timezone(&Local).format("%d.%m %H:%M").to_string())
        .unwrap_or_else(|| "неизвестно".into())
}

unsafe fn create_system_font(hwnd: HWND, semibold: bool) -> HFONT {
    unsafe {
        let dpi = GetDpiForWindow(hwnd).max(96);
        let mut metrics = NONCLIENTMETRICSW {
            cbSize: size_of::<NONCLIENTMETRICSW>() as u32,
            ..Default::default()
        };
        if SystemParametersInfoForDpi(
            SPI_GETNONCLIENTMETRICS.0,
            metrics.cbSize,
            Some(&mut metrics as *mut NONCLIENTMETRICSW as *mut c_void),
            Default::default(),
            dpi,
        )
        .is_err()
        {
            metrics.lfMessageFont.lfHeight = -scale(hwnd, 12);
            copy_wide("Segoe UI", &mut metrics.lfMessageFont.lfFaceName);
        }
        if semibold {
            metrics.lfMessageFont.lfWeight = FW_SEMIBOLD.0 as i32;
        }
        CreateFontIndirectW(&metrics.lfMessageFont)
    }
}

unsafe fn select_font(hdc: windows::Win32::Graphics::Gdi::HDC, font: HFONT) {
    unsafe {
        let _ = SelectObject(hdc, HGDIOBJ(font.0));
    }
}

unsafe fn text_out(hdc: windows::Win32::Graphics::Gdi::HDC, x: i32, y: i32, text: &str) {
    unsafe {
        let text = wide(text);
        let _ = TextOutW(hdc, x, y, &text[..text.len() - 1]);
    }
}

unsafe fn fill_rect(hdc: windows::Win32::Graphics::Gdi::HDC, rect: &RECT, color: COLORREF) {
    unsafe {
        let brush = CreateSolidBrush(color);
        FillRect(hdc, rect, brush);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }
}

unsafe fn fill_rounded_rect(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: &RECT,
    color: COLORREF,
    background: COLORREF,
    radius: i32,
) {
    unsafe {
        let radius = radius
            .max(1)
            .min((rect.right - rect.left) / 2)
            .min((rect.bottom - rect.top) / 2);
        fill_rect(
            hdc,
            &RECT {
                left: rect.left + radius,
                right: rect.right - radius,
                ..*rect
            },
            color,
        );
        fill_rect(
            hdc,
            &RECT {
                top: rect.top + radius,
                bottom: rect.bottom - radius,
                ..*rect
            },
            color,
        );

        const SAMPLES: i32 = 4;
        for corner_y in 0..radius {
            for corner_x in 0..radius {
                let mut coverage = 0;
                for sample_y in 0..SAMPLES {
                    for sample_x in 0..SAMPLES {
                        let x = corner_x as f32 + (sample_x as f32 + 0.5) / SAMPLES as f32;
                        let y = corner_y as f32 + (sample_y as f32 + 0.5) / SAMPLES as f32;
                        let dx = radius as f32 - x;
                        let dy = radius as f32 - y;
                        coverage += (dx * dx + dy * dy <= (radius * radius) as f32) as u32;
                    }
                }
                let blended = blend_color(
                    background,
                    color,
                    coverage as f32 / (SAMPLES * SAMPLES) as f32,
                );
                for (x, y) in [
                    (rect.left + corner_x, rect.top + corner_y),
                    (rect.right - 1 - corner_x, rect.top + corner_y),
                    (rect.left + corner_x, rect.bottom - 1 - corner_y),
                    (rect.right - 1 - corner_x, rect.bottom - 1 - corner_y),
                ] {
                    let _ = SetPixelV(hdc, x, y, blended);
                }
            }
        }
    }
}

fn blend_color(background: COLORREF, foreground: COLORREF, alpha: f32) -> COLORREF {
    let channel = |shift: u32| {
        let background = ((background.0 >> shift) & 0xff) as f32;
        let foreground = ((foreground.0 >> shift) & 0xff) as f32;
        (background + (foreground - background) * alpha).round() as u8
    };
    rgb(channel(0), channel(8), channel(16))
}

fn remaining_color(percent: u32) -> COLORREF {
    if percent <= 10 {
        rgb(255, 91, 110)
    } else if percent <= 30 {
        rgb(255, 181, 71)
    } else {
        windows_accent_color().unwrap_or_else(|| rgb(0, 120, 212))
    }
}

fn windows_accent_color() -> Option<COLORREF> {
    unsafe {
        let mut argb = 0_u32;
        let mut opaque = BOOL::default();
        DwmGetColorizationColor(&mut argb, &mut opaque).ok()?;
        Some(rgb(
            ((argb >> 16) & 0xff) as u8,
            ((argb >> 8) & 0xff) as u8,
            (argb & 0xff) as u8,
        ))
    }
}

fn status_icon_resource(percent: u32) -> u16 {
    let (resource, _) = STATUS_ICON_RESOURCES
        .iter()
        .map(|&(level, resource)| (resource, level.abs_diff(percent)))
        .min_by_key(|&(_, distance)| distance)
        .expect("status icon states are defined");
    resource
}

unsafe fn load_status_icon(hwnd: HWND, resource: u16) -> WinResult<HICON> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let dpi = GetDpiForWindow(hwnd).max(96);
        let size = GetSystemMetricsForDpi(SM_CXSMICON, dpi);
        let handle = LoadImageW(
            Some(HINSTANCE(instance.0)),
            resource_name(resource),
            IMAGE_ICON,
            size,
            size,
            LR_DEFAULTCOLOR,
        )?;
        Ok(HICON(handle.0))
    }
}

const fn resource_name(resource: u16) -> PCWSTR {
    PCWSTR(resource as usize as *const u16)
}

unsafe fn scale(hwnd: HWND, logical_pixels: i32) -> i32 {
    unsafe {
        let dpi = GetDpiForWindow(hwnd).max(96) as i64;
        ((logical_pixels as i64 * dpi + 48) / 96) as i32
    }
}

const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF(red as u32 | ((green as u32) << 8) | ((blue as u32) << 16))
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn copy_wide<const N: usize>(value: &str, target: &mut [u16; N]) {
    target.fill(0);
    for (destination, source) in target
        .iter_mut()
        .take(N.saturating_sub(1))
        .zip(value.encode_utf16())
    {
        *destination = source;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_common_windows() {
        assert_eq!(format_window(Some(300)), "5 ч");
        assert_eq!(format_window(Some(10_080)), "1 нед.");
    }

    #[test]
    fn chooses_nearest_remaining_quota_icon() {
        assert_eq!(status_icon_resource(0), 100);
        assert_eq!(status_icon_resource(3), 101);
        assert_eq!(status_icon_resource(24), 102);
        assert_eq!(status_icon_resource(97), 105);
        assert_eq!(status_icon_resource(100), 106);
    }

    #[test]
    fn classifies_account_and_subscription_failures() {
        assert_eq!(
            classify_error("Unauthorized: login required"),
            UiStatus::AccountRequired
        );
        assert_eq!(
            classify_error("Subscription payment required"),
            UiStatus::SubscriptionRequired
        );
        assert_eq!(
            classify_error("не удалось запустить codex: файл не найден"),
            UiStatus::CodexMissing
        );
        assert_eq!(classify_error("app-server crashed"), UiStatus::Error);
    }

    #[test]
    fn quotes_autostart_executable_path() {
        assert_eq!(
            quote_executable(Path::new(r"C:\Portable Apps\Codex Tray\codex-tray.exe")),
            r#""C:\Portable Apps\Codex Tray\codex-tray.exe""#
        );
    }
}

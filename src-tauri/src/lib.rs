use std::str::FromStr;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(serde::Serialize, Clone, Debug)]
pub struct ShortcutStatus {
    pub shortcut: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct AppState {
    pub shortcuts: Mutex<Vec<ShortcutStatus>>,
}

#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

#[tauri::command]
fn toggle_always_on_top(window: tauri::WebviewWindow) -> Result<bool, String> {
    let current = window.is_always_on_top().map_err(|e| e.to_string())?;
    let new_state = !current;
    window.set_always_on_top(new_state).map_err(|e| e.to_string())?;
    Ok(new_state)
}

#[tauri::command]
fn get_always_on_top(window: tauri::WebviewWindow) -> Result<bool, String> {
    window.is_always_on_top().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_shortcut_diagnostics(state: tauri::State<AppState>) -> Vec<ShortcutStatus> {
    state.shortcuts.lock().map(|s| s.clone()).unwrap_or_default()
}

fn toggle_window(window: &tauri::WebviewWindow) {
    if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = window.emit("summon", ());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            toggle_window(&window);
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            hide_window,
            toggle_always_on_top,
            get_always_on_top,
            get_shortcut_diagnostics
        ])
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            let start_minimized = args.iter().any(|arg| {
                arg == "--minimized" || arg == "--hidden" || arg == "-m"
            });

            if !start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }

            // Register exactly one global summon shortcut.
            // Primary: Win+Shift+T. If that fails, fall back to Alt+Shift+T.
            let global_shortcut = app.global_shortcut();
            let candidates = ["super+shift+t", "alt+shift+t"];
            let mut statuses = Vec::new();
            let mut registered = false;

            for s in candidates {
                let parsed = match Shortcut::from_str(s) {
                    Ok(sc) => sc,
                    Err(e) => {
                        eprintln!("[Trace] Invalid shortcut syntax {}: {}", s, e);
                        statuses.push(ShortcutStatus {
                            shortcut: s.to_string(),
                            success: false,
                            error: Some(e.to_string()),
                        });
                        continue;
                    }
                };

                match global_shortcut.register(parsed) {
                    Ok(_) => {
                        statuses.push(ShortcutStatus {
                            shortcut: s.to_string(),
                            success: true,
                            error: None,
                        });
                        registered = true;
                        break; // one is enough
                    }
                    Err(e) => {
                        eprintln!("[Trace] Failed to register {}: {}", s, e);
                        statuses.push(ShortcutStatus {
                            shortcut: s.to_string(),
                            success: false,
                            error: Some(e.to_string()),
                        });
                    }
                }
            }

            if !registered {
                eprintln!("[Trace] No global summon shortcut could be registered");
            }

            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(mut guard) = state.shortcuts.lock() {
                    *guard = statuses;
                }
            }

            let show_item = MenuItem::with_id(app, "show", "Open Trace", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let tray_builder = TrayIconBuilder::with_id("tray")
                .icon(tauri::include_image!("icons/icon.ico"))
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Trace")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                            let _ = window.emit("summon", ());
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            toggle_window(&window);
                        }
                    }
                });

            // If the tray fails, the app could be invisible and unreachable.
            // Force the window visible as a safety net.
            if let Err(e) = tray_builder.build(app) {
                eprintln!("[Trace] Tray icon failed to build: {}", e);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to start Trace");
}

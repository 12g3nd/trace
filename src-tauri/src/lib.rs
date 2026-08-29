mod media;
mod sidecar;

use std::str::FromStr;
use std::sync::{Arc, Mutex};
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
    launchers: Arc<sidecar::LauncherRegistry>,
    media: Arc<Mutex<media::MediaController>>,
}

#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) {
    let _ = window.hide();
}

#[tauri::command]
fn toggle_always_on_top(window: tauri::WebviewWindow) -> Result<bool, String> {
    let current = window.is_always_on_top().map_err(|e| e.to_string())?;
    let new_state = !current;
    window
        .set_always_on_top(new_state)
        .map_err(|e| e.to_string())?;
    Ok(new_state)
}

#[tauri::command]
fn get_always_on_top(window: tauri::WebviewWindow) -> Result<bool, String> {
    window.is_always_on_top().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_shortcut_diagnostics(state: tauri::State<AppState>) -> Vec<ShortcutStatus> {
    state
        .shortcuts
        .lock()
        .map(|s| s.clone())
        .unwrap_or_default()
}

#[tauri::command]
fn show_trace(app: tauri::AppHandle) -> Result<(), String> {
    show_main_window(&app)
}

#[tauri::command]
fn launch_app(app: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.launchers.launch(&app)
}

#[tauri::command]
async fn is_localsend_running() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(sidecar::is_localsend_running)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_media_state(
    artwork_key: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<media::MediaState, String> {
    let media = Arc::clone(&state.media);
    tauri::async_runtime::spawn_blocking(move || {
        media
            .lock()
            .map_err(|_| "media controller lock is unavailable".to_string())?
            .state(artwork_key.as_deref())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn media_command(action: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let media = Arc::clone(&state.media);
    tauri::async_runtime::spawn_blocking(move || {
        media
            .lock()
            .map_err(|_| "media controller lock is unavailable".to_string())?
            .execute(&action)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn reanchor_sidecar(app: tauri::AppHandle) -> Result<(), String> {
    sidecar::anchor(&app)
}

#[tauri::command]
fn toggle_sidecar(app: tauri::AppHandle) -> Result<bool, String> {
    sidecar::toggle(&app)
}

fn show_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main Trace window is unavailable".to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.unminimize().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    window.emit("summon", ()).map_err(|error| error.to_string())
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
            get_shortcut_diagnostics,
            show_trace,
            launch_app,
            is_localsend_running,
            get_media_state,
            media_command,
            reanchor_sidecar,
            toggle_sidecar
        ])
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            let start_minimized = args
                .iter()
                .any(|arg| arg == "--minimized" || arg == "--hidden" || arg == "-m");

            if !start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }

            // The Sidecar is persistent even when the main window starts hidden.
            if let Err(error) = sidecar::show(app.handle()) {
                eprintln!("[Trace] Failed to show Orbit Sidecar: {error}");
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
            let sidecar_item = MenuItem::with_id(
                app,
                "toggle-sidecar",
                "Show/Hide Sidecar",
                true,
                None::<&str>,
            )?;
            let reanchor_item = MenuItem::with_id(
                app,
                "reanchor-sidecar",
                "Re-anchor Sidecar",
                true,
                None::<&str>,
            )?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&show_item, &sidecar_item, &reanchor_item, &quit_item],
            )?;

            let tray_builder = TrayIconBuilder::with_id("tray")
                .icon(tauri::include_image!("icons/icon.ico"))
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Trace")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Err(error) = show_main_window(app) {
                            eprintln!("[Trace] Failed to show main window: {error}");
                        }
                    }
                    "toggle-sidecar" => {
                        if let Err(error) = sidecar::toggle(app) {
                            eprintln!("[Trace] Failed to toggle Sidecar: {error}");
                        }
                    }
                    "reanchor-sidecar" => {
                        if let Err(error) = sidecar::anchor(app) {
                            eprintln!("[Trace] Failed to re-anchor Sidecar: {error}");
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
            if window.label() == sidecar::SIDECAR_LABEL
                && matches!(event, tauri::WindowEvent::ScaleFactorChanged { .. })
            {
                if let Err(error) = sidecar::anchor(window.app_handle()) {
                    eprintln!("[Trace] Failed to re-anchor Sidecar after display change: {error}");
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to start Trace");
}

mod load;
mod media;
mod sidecar;

use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    Emitter, Manager,
};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

const MAIN_DISMISS_EVENT: &str = "main-dismiss-requested";
const MAIN_QUIT_EVENT: &str = "main-quit-requested";

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
    load: Arc<Mutex<load::LoadSampler>>,
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
async fn launch_app(app: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let launchers = Arc::clone(&state.launchers);
    tauri::async_runtime::spawn_blocking(move || launchers.launch(&app))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn get_launcher_state(
    state: tauri::State<'_, AppState>,
) -> Result<sidecar::LauncherState, String> {
    let launchers = Arc::clone(&state.launchers);
    tauri::async_runtime::spawn_blocking(move || launchers.state())
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
async fn open_media_source(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let media = Arc::clone(&state.media);
    tauri::async_runtime::spawn_blocking(move || {
        media
            .lock()
            .map_err(|_| "media controller lock is unavailable".to_string())?
            .open_source()
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
async fn get_load_state(state: tauri::State<'_, AppState>) -> Result<load::LoadState, String> {
    let load = Arc::clone(&state.load);
    tauri::async_runtime::spawn_blocking(move || {
        load.lock()
            .map_err(|_| "system load sampler lock is unavailable".to_string())?
            .sample()
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn open_task_manager() -> Result<(), String> {
    load::open_task_manager()
}

#[tauri::command]
async fn reanchor_sidecar(app: tauri::AppHandle) -> Result<(), String> {
    sidecar::anchor(&app)
}

#[tauri::command]
fn show_media_popover(app: tauri::AppHandle) -> Result<(), String> {
    sidecar::show_media_popover(&app)
}

#[tauri::command]
fn hide_media_popover(app: tauri::AppHandle) -> Result<(), String> {
    sidecar::hide_media_popover(&app)
}

#[tauri::command]
async fn toggle_sidecar(app: tauri::AppHandle) -> Result<bool, String> {
    sidecar::toggle(&app)
}

#[tauri::command]
async fn orbit_glass(window: tauri::WebviewWindow, previous: Option<bool>) -> Result<bool, String> {
    let enabled = std::env::var_os("LOCALAPPDATA").is_some_and(|root| {
        std::path::PathBuf::from(root).join("Orbit Noir Shelf").join("orbit-glass").is_file()
    });
    #[cfg(windows)]
    if sidecar::is_sidecar_label(window.label()) && previous != Some(enabled) {
        sidecar::set_wallpaper_blur(&window, enabled)?;
    }
    Ok(enabled)
}

/// The current wallpaper scene's accent colour, when "scene accents" is switched on in Orbit.
/// The scene comes from the state the Command Palette Dock extension publishes; the colours come
/// from the shared orbit-commands.json. Any missing piece simply means "no accent".
#[tauri::command]
fn orbit_scene_accent() -> Option<String> {
    let local = std::path::PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
    let shelf = local.join("Orbit Noir Shelf");
    if !shelf.join("scene-accents").is_file() {
        return None;
    }
    let state = std::fs::read_dir(local.join("Packages"))
        .ok()?
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("OrbitNoir.CommandPalette_"))
        .map(|entry| entry.path().join("LocalCache/Local/Orbit Noir Shelf/wallpaper-state.json"))
        .find(|path| path.is_file())?;
    let state: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(state).ok()?).ok()?;
    let commands: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(shelf.join("orbit-commands.json")).ok()?).ok()?;
    let accent = commands["scenes"][state["Scene"].as_str()?]["accent"].as_str()?;
    let valid = accent.len() == 7 && accent.starts_with('#') && accent[1..].chars().all(|c| c.is_ascii_hexdigit());
    valid.then(|| accent.to_string())
}

#[tauri::command]
fn open_orbit_panel(panel: String) -> Result<(), String> {
    if !matches!(panel.as_str(), "instruments" | "scene") {
        return Err("Unknown Orbit panel.".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let root = std::env::var_os("LOCALAPPDATA").ok_or("Local AppData is unavailable.")?;
        let shelf = std::path::PathBuf::from(root).join("Orbit Noir Shelf");
        // The Orbit host owns the shared verbs (orbit-commands.json); older installs only have the script.
        let host = shelf.join("OrbitShelf.exe");
        if host.is_file() && shelf.join("orbit-commands.json").is_file() {
            std::process::Command::new(host)
                .args(["panel", panel.as_str()])
                .creation_flags(0x08000000)
                .spawn()
                .map_err(|error| error.to_string())?;
            return Ok(());
        }
        let controls = shelf.join("OrbitControls.ps1");
        if !controls.is_file() {
            return Err("Orbit desktop controls are not installed.".into());
        }
        let windows = std::env::var_os("SystemRoot").ok_or("Windows system folder is unavailable.")?;
        let powershell = std::path::PathBuf::from(windows).join("System32/WindowsPowerShell/v1.0/powershell.exe");
        std::process::Command::new(powershell)
            .args(["-NoProfile", "-NonInteractive", "-STA", "-File"])
            .arg(controls)
            .args(["-Panel", panel.as_str()])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|error| error.to_string())?;
        Ok(())
    }
    #[cfg(not(windows))]
    Err("Orbit desktop controls require Windows.".into())
}
#[tauri::command]
fn show_sidecar_menu(window: tauri::WebviewWindow) -> Result<(), String> {
    if !sidecar::is_sidecar_label(window.label()) {
        return Err("the Sidecar menu is only available from the Sidecar".into());
    }

    let app = window.app_handle();
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let open_item = MenuItem::with_id(app, "sidecar-open", "Open Trace", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let reanchor_item = MenuItem::with_id(
        app,
        "sidecar-reanchor",
        "Re-anchor Sidecar",
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let autostart_item = CheckMenuItem::with_id(
        app,
        "sidecar-autostart",
        "Start with Windows",
        true,
        autostart_enabled,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let quit_item = MenuItem::with_id(app, "sidecar-quit", "Quit Trace", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let menu = Menu::with_items(
        app,
        &[&open_item, &reanchor_item, &autostart_item, &quit_item],
    )
    .map_err(|error| error.to_string())?;

    window.popup_menu(&menu).map_err(|error| error.to_string())
}

#[tauri::command]
fn quit_trace(app: tauri::AppHandle) {
    app.exit(0);
}

fn request_quit(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) && window.emit(MAIN_QUIT_EVENT, ()).is_ok() {
            return;
        }
    }

    app.exit(0);
}

fn show_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main Trace window is unavailable".to_string())?;
    sidecar::apply_tool_window_style(&window)?;
    window.show().map_err(|error| error.to_string())?;
    window.unminimize().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    window.emit("summon", ()).map_err(|error| error.to_string())
}

fn launch_starts_minimized(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--minimized" || arg == "--hidden" || arg == "-m")
}

fn handle_second_instance(app: &tauri::AppHandle, args: Vec<String>) {
    if let Err(error) = sidecar::show(app) {
        eprintln!("[Trace] Failed to restore Sidecar for second launch: {error}");
    }

    if !launch_starts_minimized(&args) {
        if let Err(error) = show_main_window(app) {
            eprintln!("[Trace] Failed to show main window for second launch: {error}");
        }
    }
}

fn toggle_window(window: &tauri::WebviewWindow) {
    if window.is_visible().unwrap_or(false) && window.is_focused().unwrap_or(false) {
        let _ = window.emit(MAIN_DISMISS_EVENT, ());
    } else {
        let _ = sidecar::apply_tool_window_style(window);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = window.emit("summon", ());
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Tauri requires the single-instance plugin to be registered first.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let app = app.clone();
            tauri::async_runtime::spawn_blocking(move || handle_second_instance(&app, args));
        }))
        .manage(AppState::default())
        .manage(sidecar::DisplaySync::default())
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
            get_launcher_state,
            get_media_state,
            media_command,
            open_media_source,
            get_load_state,
            open_task_manager,
            reanchor_sidecar,
            show_media_popover,
            hide_media_popover,
            toggle_sidecar,
            show_sidecar_menu,
            open_orbit_panel,
            orbit_scene_accent,
            orbit_glass,
            quit_trace
        ])
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            let start_minimized = launch_starts_minimized(&args);

            if !start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = sidecar::apply_tool_window_style(&window);
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }

            // The Sidecar is persistent even when the main window starts hidden.
            if let Err(error) = sidecar::show(app.handle()) {
                eprintln!("[Trace] Failed to show Orbit Sidecar: {error}");
            }
            #[cfg(windows)]
            sidecar::watch_taskbar_layer(app.handle().clone())?;

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

            Ok(())
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "sidecar-open" => {
                if let Err(error) = show_main_window(app) {
                    eprintln!("[Trace] Failed to show main window: {error}");
                }
            }
            "sidecar-reanchor" => {
                let app = app.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    if let Err(error) = sidecar::anchor(&app) {
                        eprintln!("[Trace] Failed to re-anchor Sidecar: {error}");
                    }
                });
            }
            "sidecar-autostart" => {
                let manager = app.autolaunch();
                let result = match manager.is_enabled() {
                    Ok(true) => manager.disable(),
                    Ok(false) => manager.enable(),
                    Err(error) => Err(error),
                };
                if let Err(error) = result {
                    eprintln!("[Trace] Failed to toggle Start with Windows: {error}");
                }
            }
            "sidecar-quit" => request_quit(app),
            _ => {}
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.emit(MAIN_DISMISS_EVENT, ());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to start Trace");
}

#[cfg(test)]
mod tests {
    use super::launch_starts_minimized;

    #[test]
    fn recognizes_all_minimized_launch_flags_in_any_argument_position() {
        for flag in ["--minimized", "--hidden", "-m"] {
            let args = vec!["trace.exe".to_string(), flag.to_string()];
            assert!(launch_starts_minimized(&args));
        }
    }

    #[test]
    fn treats_an_ordinary_launch_as_interactive() {
        let args = vec!["trace.exe".to_string()];
        assert!(!launch_starts_minimized(&args));
    }
}

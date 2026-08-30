use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Mutex,
};

use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST,
    SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_EX_APPWINDOW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

pub const SIDECAR_LABEL: &str = "sidecar";
pub const SIDECAR_WIDTH_LOGICAL: u32 = 288;
pub const SIDECAR_HEIGHT_LOGICAL: u32 = 44;
pub const LEFT_MARGIN_LOGICAL: i32 = 5;
pub const BOTTOM_MARGIN_LOGICAL: i32 = 4;
pub const MEDIA_POPOVER_LABEL: &str = "media-popover";
pub const MEDIA_POPOVER_WIDTH_LOGICAL: u32 = 288;
pub const MEDIA_POPOVER_HEIGHT_LOGICAL: u32 = 108;
pub const MEDIA_POPOVER_GAP_LOGICAL: i32 = 6;
// Stable package-family application identities discovered from this machine's
// registered Start apps. Versioned WindowsApps paths are deliberately avoided.
const CHATGPT_APP_ID: &str = "OpenAI.Codex_2p2nqsd0c76g0!App";
const CLAUDE_APP_ID: &str = "Claude_pzs8sxrjxfjjc!Claude";
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Debug)]
enum LaunchTarget {
    Executable(PathBuf),
    AppId(String),
}

#[derive(Default)]
pub struct LauncherRegistry {
    targets: Mutex<Option<HashMap<String, LaunchTarget>>>,
}

impl LauncherRegistry {
    fn targets(&self) -> HashMap<String, LaunchTarget> {
        let mut guard = self
            .targets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(targets) = guard.as_ref() {
            return targets.clone();
        }

        let targets = discover_launchers();
        *guard = Some(targets.clone());
        targets
    }

    pub fn launch(&self, app: &str) -> Result<(), String> {
        let target = self
            .targets()
            .get(app)
            .cloned()
            .ok_or_else(|| format!("{app} is not installed or could not be discovered"))?;

        let mut command = match target {
            LaunchTarget::Executable(path) => Command::new(path),
            LaunchTarget::AppId(app_id) => {
                let mut command = Command::new("explorer.exe");
                command.arg(format!("shell:AppsFolder\\{app_id}"));
                command
            }
        };

        configure_background_command(&mut command);
        command
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to launch {app}: {error}"))
    }
}

fn discover_launchers() -> HashMap<String, LaunchTarget> {
    let mut targets = HashMap::new();

    if let Some(path) = discover_localsend_executable() {
        targets.insert("localsend".into(), LaunchTarget::Executable(path));
    }

    targets.insert("chatgpt".into(), LaunchTarget::AppId(CHATGPT_APP_ID.into()));
    targets.insert("claude".into(), LaunchTarget::AppId(CLAUDE_APP_ID.into()));

    targets
}

fn discover_localsend_executable() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    for variable in ["ProgramW6432", "ProgramFiles"] {
        if let Some(root) = env::var_os(variable) {
            candidates.push(Path::new(&root).join("LocalSend").join("localsend_app.exe"));
        }
    }

    if let Some(root) = env::var_os("LOCALAPPDATA") {
        candidates.push(
            Path::new(&root)
                .join("Programs")
                .join("LocalSend")
                .join("localsend_app.exe"),
        );
        candidates.push(
            Path::new(&root)
                .join("Programs")
                .join("localsend_app")
                .join("localsend_app.exe"),
        );
    }

    candidates.into_iter().find(|path| path.is_file())
}

fn configure_background_command(command: &mut Command) {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
}

pub fn anchor(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(SIDECAR_LABEL)
        .ok_or_else(|| "sidecar window is unavailable".to_string())?;
    window
        .set_size(LogicalSize::new(
            SIDECAR_WIDTH_LOGICAL as f64,
            SIDECAR_HEIGHT_LOGICAL as f64,
        ))
        .map_err(|error| error.to_string())?;
    let monitor = window
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "primary monitor is unavailable".to_string())?;
    let window_size = window.outer_size().map_err(|error| error.to_string())?;
    // Monitor position/size are the physical full bounds. Deliberately do not
    // use work_area(): the Orbit Rail occupies the physical bottom screen band.
    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let scale = monitor.scale_factor();
    let left_margin = (LEFT_MARGIN_LOGICAL as f64 * scale).round() as i32;
    let bottom_margin = (BOTTOM_MARGIN_LOGICAL as f64 * scale).round() as i32;
    let (x, y) = anchored_position(
        monitor_position.x,
        monitor_position.y,
        monitor_size.width,
        monitor_size.height,
        window_size.width,
        window_size.height,
        left_margin,
        bottom_margin,
    );

    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|error| error.to_string())?;
    anchor_visible_media_popover(app)?;
    ensure_topmost(&window)
}

fn anchor_visible_media_popover(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(MEDIA_POPOVER_LABEL) else {
        return Ok(());
    };
    if window.is_visible().map_err(|error| error.to_string())? {
        anchor_media_popover(app)?;
    }
    Ok(())
}

fn anchor_media_popover(app: &AppHandle) -> Result<(), String> {
    let sidecar = app
        .get_webview_window(SIDECAR_LABEL)
        .ok_or_else(|| "sidecar window is unavailable".to_string())?;
    let popover = app
        .get_webview_window(MEDIA_POPOVER_LABEL)
        .ok_or_else(|| "media popover window is unavailable".to_string())?;
    popover
        .set_size(LogicalSize::new(
            MEDIA_POPOVER_WIDTH_LOGICAL as f64,
            MEDIA_POPOVER_HEIGHT_LOGICAL as f64,
        ))
        .map_err(|error| error.to_string())?;

    let sidecar_position = sidecar
        .outer_position()
        .map_err(|error| error.to_string())?;
    let popover_size = popover.outer_size().map_err(|error| error.to_string())?;
    let scale = sidecar.scale_factor().map_err(|error| error.to_string())?;
    let gap = (MEDIA_POPOVER_GAP_LOGICAL as f64 * scale).round() as i32;
    let screen_top = sidecar
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .map(|monitor| monitor.position().y)
        .unwrap_or(i32::MIN);
    let (x, y) = media_popover_position(
        sidecar_position.x,
        sidecar_position.y,
        popover_size.height,
        gap,
        screen_top,
    );

    popover
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|error| error.to_string())?;
    ensure_topmost(&popover)
}

pub fn show(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(SIDECAR_LABEL)
        .ok_or_else(|| "sidecar window is unavailable".to_string())?;

    // Reapplying SWP_FRAMECHANGED to an already-visible WebView can remove its
    // native surface. Re-anchor is the idempotent restore path for repeat launches.
    if window.is_visible().map_err(|error| error.to_string())? {
        return anchor(app);
    }

    apply_tool_window_style(&window)?;
    anchor(app)?;
    window.show().map_err(|error| error.to_string())?;
    ensure_topmost(&window)
}

pub fn show_media_popover(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MEDIA_POPOVER_LABEL)
        .ok_or_else(|| "media popover window is unavailable".to_string())?;

    if window.is_visible().map_err(|error| error.to_string())? {
        return anchor_media_popover(app);
    }

    apply_nonactivating_tool_window_style(&window)?;
    anchor_media_popover(app)?;
    window.show().map_err(|error| error.to_string())?;
    ensure_topmost(&window)
}

pub fn hide_media_popover(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MEDIA_POPOVER_LABEL)
        .ok_or_else(|| "media popover window is unavailable".to_string())?;
    window.hide().map_err(|error| error.to_string())
}

#[cfg(windows)]
fn ensure_topmost(window: &tauri::WebviewWindow) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;

    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[cfg(not(windows))]
fn ensure_topmost(window: &tauri::WebviewWindow) -> Result<(), String> {
    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())
}

#[cfg(windows)]
pub fn apply_tool_window_style(window: &tauri::WebviewWindow) -> Result<(), String> {
    apply_shell_window_style(window, false)
}

#[cfg(windows)]
fn apply_nonactivating_tool_window_style(window: &tauri::WebviewWindow) -> Result<(), String> {
    apply_shell_window_style(window, true)
}

#[cfg(windows)]
fn apply_shell_window_style(
    window: &tauri::WebviewWindow,
    nonactivating: bool,
) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;

    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let shell_style = if nonactivating {
            nonactivating_tool_window_ex_style(current)
        } else {
            tool_window_ex_style(current)
        };
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, shell_style as isize);
        SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[cfg(windows)]
fn tool_window_ex_style(current: u32) -> u32 {
    (current | WS_EX_TOOLWINDOW.0) & !WS_EX_APPWINDOW.0
}

#[cfg(windows)]
fn nonactivating_tool_window_ex_style(current: u32) -> u32 {
    tool_window_ex_style(current) | WS_EX_NOACTIVATE.0
}

#[cfg(not(windows))]
pub fn apply_tool_window_style(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
fn apply_nonactivating_tool_window_style(_window: &tauri::WebviewWindow) -> Result<(), String> {
    Ok(())
}

pub fn toggle(app: &AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window(SIDECAR_LABEL)
        .ok_or_else(|| "sidecar window is unavailable".to_string())?;

    if window.is_visible().map_err(|error| error.to_string())? {
        let _ = hide_media_popover(app);
        window.hide().map_err(|error| error.to_string())?;
        Ok(false)
    } else {
        show(app)?;
        Ok(true)
    }
}

fn anchored_position(
    screen_x: i32,
    screen_y: i32,
    screen_width: u32,
    screen_height: u32,
    window_width: u32,
    window_height: u32,
    left_margin: i32,
    bottom_margin: i32,
) -> (i32, i32) {
    let max_x = screen_x.saturating_add(screen_width.saturating_sub(window_width) as i32);
    let max_y = screen_y.saturating_add(screen_height.saturating_sub(window_height) as i32);
    let x = screen_x
        .saturating_add(left_margin)
        .min(max_x)
        .max(screen_x);
    let y = max_y.saturating_sub(bottom_margin).max(screen_y);
    (x, y)
}

fn media_popover_position(
    sidecar_x: i32,
    sidecar_y: i32,
    popover_height: u32,
    gap: i32,
    screen_top: i32,
) -> (i32, i32) {
    let y = sidecar_y
        .saturating_sub(popover_height as i32)
        .saturating_sub(gap)
        .max(screen_top);
    (sidecar_x, y)
}

#[cfg(test)]
mod tests {
    use super::{
        anchored_position, media_popover_position, BOTTOM_MARGIN_LOGICAL, LEFT_MARGIN_LOGICAL,
        MEDIA_POPOVER_GAP_LOGICAL, MEDIA_POPOVER_HEIGHT_LOGICAL, SIDECAR_HEIGHT_LOGICAL,
        SIDECAR_WIDTH_LOGICAL,
    };

    #[test]
    fn anchors_inside_the_bottom_left_of_the_full_monitor_bounds() {
        assert_eq!(
            anchored_position(
                0,
                0,
                1920,
                1080,
                SIDECAR_WIDTH_LOGICAL,
                SIDECAR_HEIGHT_LOGICAL,
                LEFT_MARGIN_LOGICAL,
                BOTTOM_MARGIN_LOGICAL,
            ),
            (5, 1032)
        );
    }

    #[test]
    fn respects_offset_monitor_coordinates_and_clamps_small_monitors() {
        assert_eq!(
            anchored_position(
                -1280,
                40,
                1280,
                1024,
                SIDECAR_WIDTH_LOGICAL,
                SIDECAR_HEIGHT_LOGICAL,
                LEFT_MARGIN_LOGICAL,
                BOTTOM_MARGIN_LOGICAL,
            ),
            (-1275, 1016)
        );
        assert_eq!(
            anchored_position(
                10,
                20,
                100,
                40,
                SIDECAR_WIDTH_LOGICAL,
                SIDECAR_HEIGHT_LOGICAL,
                LEFT_MARGIN_LOGICAL,
                BOTTOM_MARGIN_LOGICAL,
            ),
            (10, 20)
        );
    }

    #[test]
    fn places_media_popover_directly_above_the_actual_sidecar_rectangle() {
        assert_eq!(
            media_popover_position(
                8,
                1128,
                MEDIA_POPOVER_HEIGHT_LOGICAL * 3 / 2,
                MEDIA_POPOVER_GAP_LOGICAL * 3 / 2,
                0,
            ),
            (8, 957)
        );
        assert_eq!(media_popover_position(-1275, 50, 108, 6, 40), (-1275, 40));
    }

    #[cfg(windows)]
    #[test]
    fn tool_window_style_suppresses_normal_app_switching_presence() {
        use super::{nonactivating_tool_window_ex_style, tool_window_ex_style};
        use windows::Win32::UI::WindowsAndMessaging::{
            WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        };

        let style = tool_window_ex_style(WS_EX_APPWINDOW.0);
        assert_ne!(style & WS_EX_TOOLWINDOW.0, 0);
        assert_eq!(style & WS_EX_APPWINDOW.0, 0);

        let popover_style = nonactivating_tool_window_ex_style(WS_EX_APPWINDOW.0);
        assert_ne!(popover_style & WS_EX_TOOLWINDOW.0, 0);
        assert_ne!(popover_style & WS_EX_NOACTIVATE.0, 0);
        assert_eq!(popover_style & WS_EX_APPWINDOW.0, 0);
    }
}

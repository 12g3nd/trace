use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Mutex,
};

use tauri::{AppHandle, Manager, PhysicalPosition};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows::Win32::{
    Foundation::CloseHandle,
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    },
    UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    },
};

pub const SIDECAR_LABEL: &str = "sidecar";
const LEFT_MARGIN: i32 = 8;
const BOTTOM_MARGIN: i32 = 5;
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
    let left_margin = (LEFT_MARGIN as f64 * scale).round() as i32;
    let bottom_margin = (BOTTOM_MARGIN as f64 * scale).round() as i32;
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
        .map_err(|error| error.to_string())
}

pub fn show(app: &AppHandle) -> Result<(), String> {
    apply_shell_window_style(app)?;
    anchor(app)?;
    app.get_webview_window(SIDECAR_LABEL)
        .ok_or_else(|| "sidecar window is unavailable".to_string())?
        .show()
        .map_err(|error| error.to_string())
}

#[cfg(windows)]
pub fn apply_shell_window_style(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(SIDECAR_LABEL)
        .ok_or_else(|| "sidecar window is unavailable".to_string())?;
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;

    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let shell_style = (current | WS_EX_TOOLWINDOW.0) & !WS_EX_APPWINDOW.0;
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

#[cfg(not(windows))]
pub fn apply_shell_window_style(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}

pub fn toggle(app: &AppHandle) -> Result<bool, String> {
    let window = app
        .get_webview_window(SIDECAR_LABEL)
        .ok_or_else(|| "sidecar window is unavailable".to_string())?;

    if window.is_visible().map_err(|error| error.to_string())? {
        window.hide().map_err(|error| error.to_string())?;
        Ok(false)
    } else {
        anchor(app)?;
        window.show().map_err(|error| error.to_string())?;
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

#[cfg(windows)]
pub fn is_localsend_running() -> bool {
    // A native snapshot is substantially cheaper than spawning tasklist repeatedly.
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return false;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut running = false;

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let length = entry
                    .szExeFile
                    .iter()
                    .position(|character| *character == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..length]);
                if name.eq_ignore_ascii_case("localsend_app.exe") {
                    running = true;
                    break;
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
        running
    }
}

#[cfg(not(windows))]
pub fn is_localsend_running() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::anchored_position;

    #[test]
    fn anchors_inside_the_bottom_left_of_the_full_monitor_bounds() {
        assert_eq!(
            anchored_position(0, 0, 1920, 1080, 324, 48, 8, 5),
            (8, 1027)
        );
    }

    #[test]
    fn respects_offset_monitor_coordinates_and_clamps_small_monitors() {
        assert_eq!(
            anchored_position(-1280, 40, 1280, 1024, 324, 48, 8, 5),
            (-1272, 1011)
        );
        assert_eq!(anchored_position(10, 20, 100, 40, 324, 48, 8, 5), (10, 20));
    }
}

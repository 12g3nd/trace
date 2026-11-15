use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Mutex,
};

use tauri::{
    AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewWindowBuilder,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use windows::core::{w, BOOL};
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM};
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONEAREST};
#[cfg(windows)]
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows::Win32::UI::Shell::{ITaskbarList, TaskbarList};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetWindowLongPtrW, GetWindowTextLengthW,
    GetWindowThreadProcessId, IsWindowVisible, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE,
    HWND_BOTTOM, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
};

pub const SIDECAR_LABEL: &str = "sidecar";
pub const SIDECAR_WIDTH_LOGICAL: u32 = 317;
pub const SIDECAR_HEIGHT_LOGICAL: u32 = 48;
pub const LEFT_MARGIN_LOGICAL: i32 = 5;
pub const BOTTOM_MARGIN_LOGICAL: i32 = 0;
pub const MEDIA_POPOVER_LABEL: &str = "media-popover";
pub const MEDIA_POPOVER_WIDTH_LOGICAL: u32 = 288;
pub const MEDIA_POPOVER_HEIGHT_LOGICAL: u32 = 108;
pub const MEDIA_POPOVER_GAP_LOGICAL: i32 = 6;
// Stable package-family application identities discovered from this machine's
// registered Start apps. Versioned WindowsApps paths are deliberately avoided.
const CODEX_APP_ID: &str = "OpenAI.Codex_2p2nqsd0c76g0!App";
const CLAUDE_APP_ID: &str = "Claude_pzs8sxrjxfjjc!Claude";
const CODEX_PROCESS_NAME: &str = "chatgpt.exe";
const CLAUDE_PROCESS_NAME: &str = "claude.exe";
const GEMINI_PROCESS_NAME: &str = "gemini.exe";
const LOCALSEND_PROCESS_NAME: &str = "localsend_app.exe";
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherState {
    pub localsend_running: bool,
    pub codex_running: bool,
    pub claude_running: bool,
    pub gemini_running: bool,
}

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
        // Switching an existing window avoids the package activation round trip.
        #[cfg(windows)]
        if let Some(executable) = match app {
            "codex" => Some(CODEX_PROCESS_NAME),
            "claude" => Some(CLAUDE_PROCESS_NAME),
            "gemini" => Some(GEMINI_PROCESS_NAME),
            "localsend" => Some(LOCALSEND_PROCESS_NAME),
            "atlas" => Some("personal-atlas.exe"),
            _ => None,
        } {
            if crate::media::focus_running_executable(executable).is_ok() {
                return Ok(());
            }
        }
        let target = self
            .targets()
            .get(app)
            .cloned()
            .ok_or_else(|| format!("{app} is not installed or could not be discovered"))?;

        match target {
            LaunchTarget::Executable(path) => {
                let mut command = Command::new(path);
                configure_background_command(&mut command);
                command
                    .spawn()
                    .map(|_| ())
                    .map_err(|error| format!("failed to launch {app}: {error}"))
            }
            LaunchTarget::AppId(app_id) => crate::media::activate_app_user_model_id(&app_id)
                .map_err(|error| format!("failed to activate {app}: {error}")),
        }
    }

    pub fn state(&self) -> LauncherState {
        launcher_window_state()
    }
}

fn discover_launchers() -> HashMap<String, LaunchTarget> {
    let mut targets = HashMap::new();

    if let Some(path) = discover_localsend_executable() {
        targets.insert("localsend".into(), LaunchTarget::Executable(path));
    }

    if let Some(root) = env::var_os("USERPROFILE") {
        let path = Path::new(&root)
            .join("Personal Atlas")
            .join("src-tauri")
            .join("target")
            .join("release")
            .join("personal-atlas.exe");
        if path.is_file() {
            targets.insert("atlas".into(), LaunchTarget::Executable(path));
        }
    }

    targets.insert("codex".into(), LaunchTarget::AppId(CODEX_APP_ID.into()));
    targets.insert("claude".into(), LaunchTarget::AppId(CLAUDE_APP_ID.into()));
    if let Some(root) = env::var_os("LOCALAPPDATA") {
        let path = Path::new(&root)
            .join("Google")
            .join("Gemini")
            .join("Gemini.exe");
        if path.is_file() {
            targets.insert("gemini".into(), LaunchTarget::Executable(path));
        }
    }

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

#[derive(Default)]
pub struct DisplaySync(Mutex<()>);

pub fn is_sidecar_label(label: &str) -> bool {
    label == SIDECAR_LABEL
        || label.strip_prefix("sidecar-").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        })
}

#[cfg(windows)]
pub fn set_wallpaper_blur(window: &tauri::WebviewWindow, enabled: bool) -> Result<(), String> {
    use std::ffi::c_void;

    #[repr(C)]
    struct AccentPolicy {
        state: i32,
        flags: i32,
        gradient_color: u32,
        animation_id: i32,
    }

    #[repr(C)]
    struct CompositionAttribute {
        attribute: i32,
        data: *const c_void,
        size: usize,
    }

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: HWND, attribute: u32, value: *const c_void, size: u32) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(module_name: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, procedure_name: *const u8) -> *const c_void;
    }

    const WINDOW_CORNER_PREFERENCE: u32 = 33;
    const ROUND_CORNERS: i32 = 2;
    const SYSTEM_BACKDROP_TYPE: u32 = 38;
    const NO_SYSTEM_BACKDROP: i32 = 1;
    const ACCENT_POLICY: i32 = 19;
    const ACRYLIC_BLUR_BEHIND: i32 = 4;
    const ACRYLIC_NAVY: u32 = 0x281F_140C; // Alpha, blue, green, red.
    const INVALID_ARGUMENT: i32 = 0x8007_0057u32 as i32;

    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    let policy = AccentPolicy {
        state: if enabled { ACRYLIC_BLUR_BEHIND } else { 0 },
        flags: 0,
        gradient_color: ACRYLIC_NAVY,
        animation_id: 0,
    };
    let attribute = CompositionAttribute {
        attribute: ACCENT_POLICY,
        data: (&policy as *const AccentPolicy).cast(),
        size: std::mem::size_of::<AccentPolicy>(),
    };

    unsafe {
        // Windows may paint the system Acrylic material as opaque on this always-on-top
        // WebView window. The accent policy samples the scene behind it instead.
        let user32 = GetModuleHandleW(w!("user32.dll").as_ptr());
        if user32.is_null() {
            return Err("Windows composition is unavailable.".into());
        }
        let address = GetProcAddress(user32, b"SetWindowCompositionAttribute\0".as_ptr());
        if address.is_null() {
            return Err("Windows accent blur is unavailable.".into());
        }
        let set_accent: unsafe extern "system" fn(HWND, *const CompositionAttribute) -> BOOL =
            std::mem::transmute(address);
        let result = DwmSetWindowAttribute(
            hwnd,
            SYSTEM_BACKDROP_TYPE,
            (&NO_SYSTEM_BACKDROP as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
        if result < 0 && result != INVALID_ARGUMENT {
            return Err(format!("Could not clear the system backdrop ({result:#x})."));
        }
        // The Acrylic layer ignores GDI window regions. Let the compositor round its own
        // backdrop and keep the transparent WebView at the same radius.
        let corner_result = DwmSetWindowAttribute(
            hwnd,
            WINDOW_CORNER_PREFERENCE,
            (&ROUND_CORNERS as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
        if corner_result < 0 && corner_result != INVALID_ARGUMENT {
            return Err(format!("Could not round the TRACE backdrop ({corner_result:#x})."));
        }
        if !set_accent(hwnd, &attribute).as_bool() {
            return Err("Windows did not apply the TRACE accent policy.".into());
        }
    }
    Ok(())
}

// Keep the configured window on the primary display. Additional windows reuse
// the same hidden, transparent configuration and receive only shell permissions.
fn display_monitors(app: &AppHandle) -> Result<Vec<tauri::Monitor>, String> {
    let primary = app.primary_monitor().map_err(|error| error.to_string())?;
    let mut monitors = app
        .available_monitors()
        .map_err(|error| error.to_string())?;
    monitors.sort_by_key(|monitor| {
        let position = monitor.position();
        let is_primary = primary.as_ref().is_some_and(|primary| {
            primary.position() == position && primary.size() == monitor.size()
        });
        (!is_primary, position.x, position.y)
    });
    // Mirrored displays share one desktop rectangle and need only one Sidecar.
    monitors.dedup_by(|a, b| a.position() == b.position() && a.size() == b.size());
    Ok(monitors)
}

fn sidecar_label(index: usize) -> String {
    if index == 0 {
        SIDECAR_LABEL.into()
    } else {
        format!("sidecar-{index}")
    }
}

pub fn anchor(app: &AppHandle) -> Result<(), String> {
    let sync = app.state::<DisplaySync>();
    let _guard = match sync.0.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::WouldBlock) => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    let monitors = display_monitors(app)?;
    if monitors.is_empty() {
        return Ok(()); // Display enumeration can be empty briefly while docking.
    }
    let visible = app
        .get_webview_window(SIDECAR_LABEL)
        .is_some_and(|window| window.is_visible().unwrap_or(false));
    for (index, monitor) in monitors.iter().enumerate() {
        let label = sidecar_label(index);
        let window = match app.get_webview_window(&label) {
            Some(window) => window,
            None => {
                let mut config = app
                    .config()
                    .app
                    .windows
                    .iter()
                    .find(|config| config.label == SIDECAR_LABEL)
                    .ok_or_else(|| "sidecar configuration is unavailable".to_string())?
                    .clone();
                config.label = label;
                let window = WebviewWindowBuilder::from_config(app, &config)
                    .map_err(|error| error.to_string())?
                    .build()
                    .map_err(|error| error.to_string())?;
                apply_tool_window_style(&window)?;
                window
            }
        };
        anchor_window(&window, monitor)?;
        if window.is_visible().map_err(|error| error.to_string())? != visible {
            if visible {
                window.show().map_err(|error| error.to_string())?;
            } else {
                window.hide().map_err(|error| error.to_string())?;
            }
        }
        ensure_taskbar_z_order(&window)?;
    }
    // Remove disconnected displays instead of leaving invisible windows running.
    for (label, window) in app.webview_windows() {
        if is_sidecar_label(&label)
            && !(0..monitors.len()).any(|index| sidecar_label(index) == label)
        {
            window.destroy().map_err(|error| error.to_string())?;
        }
    }
    anchor_visible_media_popover(app)
}

fn sidecar_geometry(
    screen_x: i32,
    screen_y: i32,
    screen_width: u32,
    screen_height: u32,
    scale: f64,
) -> (PhysicalPosition<i32>, PhysicalSize<u32>) {
    let size = PhysicalSize::new(
        (SIDECAR_WIDTH_LOGICAL as f64 * scale).round() as u32,
        (SIDECAR_HEIGHT_LOGICAL as f64 * scale).round() as u32,
    );
    let (x, y) = anchored_position(
        screen_x,
        screen_y,
        screen_width,
        screen_height,
        size.width,
        size.height,
        (LEFT_MARGIN_LOGICAL as f64 * scale).round() as i32,
        (BOTTOM_MARGIN_LOGICAL as f64 * scale).round() as i32,
    );
    (PhysicalPosition::new(x, y), size)
}

fn anchor_window(window: &tauri::WebviewWindow, monitor: &tauri::Monitor) -> Result<(), String> {
    let (target, size) = sidecar_geometry(
        monitor.position().x,
        monitor.position().y,
        monitor.size().width,
        monitor.size().height,
        monitor.scale_factor(),
    );
    if window.outer_position().map_err(|error| error.to_string())? != target {
        window
            .set_position(target)
            .map_err(|error| error.to_string())?;
    }
    if window.outer_size().map_err(|error| error.to_string())? != size {
        window.set_size(size).map_err(|error| error.to_string())?;
    }
    Ok(())
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
        .current_monitor()
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
    ensure_taskbar_z_order(&popover)
}

pub fn show(app: &AppHandle) -> Result<(), String> {
    let primary = app
        .get_webview_window(SIDECAR_LABEL)
        .ok_or_else(|| "sidecar window is unavailable".to_string())?;
    // Do not reapply SWP_FRAMECHANGED to an already-visible WebView.
    if !primary.is_visible().map_err(|error| error.to_string())? {
        apply_tool_window_style(&primary)?;
    }
    anchor(app)?;
    for (label, window) in app.webview_windows() {
        if is_sidecar_label(&label) {
            if !window.is_visible().map_err(|error| error.to_string())? {
                window.show().map_err(|error| error.to_string())?;
            }
            ensure_taskbar_z_order(&window)?;
        }
    }
    Ok(())
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
    ensure_taskbar_z_order(&window)
}

pub fn hide_media_popover(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(MEDIA_POPOVER_LABEL)
        .ok_or_else(|| "media popover window is unavailable".to_string())?;
    window.hide().map_err(|error| error.to_string())
}

#[cfg(windows)]
fn ensure_taskbar_z_order(window: &tauri::WebviewWindow) -> Result<(), String> {
    let hwnd = window.hwnd().map_err(|error| error.to_string())?;

    unsafe {
        // Explorer drops its topmost flag for full-screen apps. Follow that
        // layer instead of unconditionally demoting Sidecar behind normal apps.
        // Default to topmost while Explorer is restarting.
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let primary = FindWindowW(w!("Shell_TrayWnd"), None).ok();
        let mut taskbar = primary
            .filter(|taskbar| MonitorFromWindow(*taskbar, MONITOR_DEFAULTTONEAREST) == monitor);
        let mut previous = None;
        while taskbar.is_none() {
            let Ok(secondary) = FindWindowExW(None, previous, w!("Shell_SecondaryTrayWnd"), None)
            else {
                break;
            };
            if MonitorFromWindow(secondary, MONITOR_DEFAULTTONEAREST) == monitor {
                taskbar = Some(secondary);
            }
            previous = Some(secondary);
        }
        let topmost = taskbar
            .map(|taskbar| GetWindowLongPtrW(taskbar, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST.0 != 0)
            .unwrap_or(true);
        sync_taskbar_layer(hwnd, topmost)
    }
}

#[cfg(windows)]
fn sync_taskbar_layer(hwnd: HWND, topmost: bool) -> Result<(), String> {
    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST.0 != 0;
        if current == topmost {
            return Ok(());
        }
        SetWindowPos(
            hwnd,
            // Demoting to the bottom also keeps us behind a full-screen app.
            Some(if topmost { HWND_TOPMOST } else { HWND_BOTTOM }),
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
fn ensure_taskbar_z_order(window: &tauri::WebviewWindow) -> Result<(), String> {
    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())
}

// Native refresh is independent of WebView background timer throttling. Only
// layer transitions call SetWindowPos, so ordinary focus changes cannot cover
// Sidecar and repeated refreshes do not raise it or steal keyboard focus.
#[cfg(windows)]
pub fn watch_taskbar_layer(app: AppHandle) -> Result<(), String> {
    std::thread::Builder::new()
        .name("sidecar-taskbar-layer".into())
        .spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if app.get_webview_window(SIDECAR_LABEL).is_none() {
                break;
            }
            // WebView2 window creation must run outside synchronous commands
            // and native event handlers. Keep display reconciliation on this
            // worker; Tauri marshals individual window operations as needed.
            if let Err(error) = anchor(&app) {
                eprintln!("[Trace] Could not sync display Sidecars: {error}");
            }
            if let Some(window) = app.get_webview_window(MEDIA_POPOVER_LABEL) {
                if let Err(error) = ensure_taskbar_z_order(&window) {
                    eprintln!("[Trace] Could not sync media taskbar layer: {error}");
                }
            }
        })
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn launcher_state_from_process_names<I, S>(process_names: I) -> LauncherState
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut state = LauncherState {
        localsend_running: false,
        codex_running: false,
        claude_running: false,
        gemini_running: false,
    };

    for process_name in process_names {
        let process_name = process_name.as_ref();
        state.codex_running |= process_name.eq_ignore_ascii_case(CODEX_PROCESS_NAME);
        state.claude_running |= process_name.eq_ignore_ascii_case(CLAUDE_PROCESS_NAME);
        state.gemini_running |= process_name.eq_ignore_ascii_case(GEMINI_PROCESS_NAME);
        state.localsend_running |= process_name.eq_ignore_ascii_case(LOCALSEND_PROCESS_NAME);
    }

    state
}

#[cfg(windows)]
fn launcher_window_state() -> LauncherState {
    let processes = tracked_processes();
    unsafe {
        let initialized = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let taskbar = if initialized {
            CoCreateInstance::<_, ITaskbarList>(&TaskbarList, None, CLSCTX_INPROC_SERVER)
                .and_then(|taskbar| {
                    taskbar.HrInit()?;
                    Ok(taskbar)
                })
                .ok()
        } else {
            None
        };
        let mut context = TaskbarContext {
            processes: &processes,
            taskbar: taskbar.as_ref(),
            window_process_ids: Vec::new(),
        };
        let _ = EnumWindows(
            Some(inspect_launcher_window),
            LPARAM(&mut context as *mut TaskbarContext<'_> as isize),
        );
        let state = launcher_state_from_windows(&processes, &context.window_process_ids);
        drop(context);
        drop(taskbar);
        if initialized {
            CoUninitialize();
        }
        state
    }
}

#[cfg(not(windows))]
fn launcher_window_state() -> LauncherState {
    launcher_state_from_windows(&HashMap::new(), &[])
}

fn launcher_state_from_windows(
    processes: &HashMap<u32, String>,
    window_process_ids: &[u32],
) -> LauncherState {
    launcher_state_from_process_names(window_process_ids.iter().filter_map(|id| processes.get(id)))
}

#[cfg(windows)]
struct TaskbarContext<'a> {
    processes: &'a HashMap<u32, String>,
    taskbar: Option<&'a ITaskbarList>,
    window_process_ids: Vec<u32>,
}

#[cfg(windows)]
fn tracked_processes() -> HashMap<u32, String> {
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return HashMap::new();
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut processes = HashMap::new();
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let length = entry
                    .szExeFile
                    .iter()
                    .position(|character| *character == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..length]);
                if [
                    CODEX_PROCESS_NAME,
                    CLAUDE_PROCESS_NAME,
                    GEMINI_PROCESS_NAME,
                    LOCALSEND_PROCESS_NAME,
                ]
                .iter()
                .any(|tracked| name.eq_ignore_ascii_case(tracked))
                {
                    processes.insert(entry.th32ProcessID, name);
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
        processes
    }
}

#[cfg(windows)]
unsafe extern "system" fn inspect_launcher_window(window: HWND, data: LPARAM) -> BOOL {
    let context = &mut *(data.0 as *mut TaskbarContext<'_>);
    let mut process_id = 0;
    GetWindowThreadProcessId(window, Some(&mut process_id));
    // IsWindowVisible includes minimized windows, but excludes close-to-tray windows.
    if !context.processes.contains_key(&process_id)
        || !IsWindowVisible(window).as_bool()
        || GetWindowTextLengthW(window) == 0
    {
        return BOOL(1);
    }
    context.window_process_ids.push(process_id);

    let current = GetWindowLongPtrW(window, GWL_EXSTYLE) as u32;
    // Repair styles applied to Claude/Codex by older Trace versions only.
    if context.processes.get(&process_id).is_some_and(|name| {
        name.eq_ignore_ascii_case(CODEX_PROCESS_NAME)
            || name.eq_ignore_ascii_case(CLAUDE_PROCESS_NAME)
    }) && current & WS_EX_TOOLWINDOW.0 != 0
    {
        SetWindowLongPtrW(
            window,
            GWL_EXSTYLE,
            ((current & !WS_EX_TOOLWINDOW.0) | WS_EX_APPWINDOW.0) as isize,
        );
    }
    if let Some(taskbar) = context.taskbar {
        if let Err(error) = taskbar.DeleteTab(window) {
            eprintln!("[Trace Sidecar] Could not remove taskbar entry: {error}");
        }
    }
    BOOL(1)
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
        for (label, window) in app.webview_windows() {
            if is_sidecar_label(&label) {
                window.hide().map_err(|error| error.to_string())?;
            }
        }
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
    #[test]
    fn mixed_dpi_displays_use_their_own_physical_size_and_desktop_origin() {
        use tauri::{PhysicalPosition, PhysicalSize};
        assert_eq!(
            super::sidecar_geometry(0, 0, 1920, 1080, 1.0),
            (PhysicalPosition::new(5, 1032), PhysicalSize::new(317, 48)),
        );
        assert_eq!(
            super::sidecar_geometry(-1920, 80, 1920, 1200, 1.5),
            (
                PhysicalPosition::new(-1912, 1208),
                PhysicalSize::new(476, 72)
            ),
        );
    }

    #[test]
    fn only_open_windows_light_launchers_even_if_background_processes_remain() {
        let processes = std::collections::HashMap::from([
            (1, "ChatGPT.exe".to_string()),
            (2, "claude.exe".to_string()),
            (3, "localsend_app.exe".to_string()),
            (4, "Gemini.exe".to_string()),
        ]);
        let state = super::launcher_state_from_windows(&processes, &[1, 2, 3, 4]);
        assert!(
            state.codex_running
                && state.claude_running
                && state.localsend_running
                && state.gemini_running
        );
        let state = super::launcher_state_from_windows(&processes, &[3]);
        assert!(!state.codex_running && !state.claude_running && state.localsend_running);
        assert!(!state.gemini_running);
        let state = super::launcher_state_from_windows(&processes, &[4]);
        assert!(
            state.gemini_running
                && !state.claude_running
                && !state.codex_running
                && !state.localsend_running
        );
        let state = super::launcher_state_from_windows(&processes, &[]);
        assert!(
            !state.codex_running
                && !state.claude_running
                && !state.localsend_running
                && !state.gemini_running
        );
    }

    use super::{
        anchored_position, launcher_state_from_process_names, media_popover_position,
        BOTTOM_MARGIN_LOGICAL, LEFT_MARGIN_LOGICAL, MEDIA_POPOVER_GAP_LOGICAL,
        MEDIA_POPOVER_HEIGHT_LOGICAL, SIDECAR_HEIGHT_LOGICAL, SIDECAR_WIDTH_LOGICAL,
    };

    #[cfg(windows)]
    #[test]
    fn native_shell_layer_follows_taskbar_transitions_without_focus_or_repeated_raising() {
        use windows::core::w;
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, DispatchMessageW, GetForegroundWindow, GetWindow,
            GetWindowLongPtrW, PeekMessageW, ShowWindow, GWL_EXSTYLE, GW_HWNDPREV, MSG, PM_REMOVE,
            SW_SHOWNOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
        };

        // Real HWNDs exercise the native API, including idempotent refreshes.
        struct TestWindow(super::HWND);
        impl Drop for TestWindow {
            fn drop(&mut self) {
                unsafe {
                    let _ = DestroyWindow(self.0);
                }
            }
        }
        unsafe {
            let window = TestWindow(
                CreateWindowExW(
                    WS_EX_TOOLWINDOW,
                    w!("STATIC"),
                    w!("Trace layer regression"),
                    WS_POPUP,
                    0,
                    0,
                    1,
                    1,
                    None,
                    None,
                    None,
                    None,
                )
                .unwrap(),
            );
            let foreground = GetForegroundWindow();
            // Keep a one-pixel surface on-screen and non-activating. Windows
            // can disregard topmost transitions for entirely off-screen windows.
            let _ = ShowWindow(window.0, SW_SHOWNOACTIVATE);
            for topmost in [true, true, false, false, true] {
                super::sync_taskbar_layer(window.0, topmost).unwrap();
                let mut message = MSG::default();
                while PeekMessageW(&mut message, None, 0, 0, PM_REMOVE).as_bool() {
                    DispatchMessageW(&message);
                }
                assert_eq!(
                    GetWindowLongPtrW(window.0, GWL_EXSTYLE) as u32 & WS_EX_TOPMOST.0 != 0,
                    topmost
                );
                let before = GetWindow(window.0, GW_HWNDPREV).ok();
                super::sync_taskbar_layer(window.0, topmost).unwrap();
                assert_eq!(GetWindow(window.0, GW_HWNDPREV).ok(), before);
                assert_eq!(GetForegroundWindow(), foreground);
            }
        }
    }

    #[test]
    fn reports_codex_and_claude_from_their_desktop_processes() {
        let state =
            launcher_state_from_process_names(["explorer.exe", "ChatGPT.exe", "claude.exe"]);

        assert!(state.codex_running);
        assert!(state.claude_running);
    }

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

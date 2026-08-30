use base64::{engine::general_purpose::STANDARD, Engine as _};
use windows::{
    core::{BOOL, PCWSTR},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionMediaProperties,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Storage::Streams::DataReader,
    Win32::{
        Foundation::{CloseHandle, HWND, LPARAM},
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_LOCAL_SERVER,
                COINIT_APARTMENTTHREADED,
            },
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
        },
        UI::{
            Shell::{ApplicationActivationManager, IApplicationActivationManager, AO_NOERRORUI},
            WindowsAndMessaging::{
                EnumWindows, GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible,
                SetForegroundWindow, ShowWindowAsync, SW_RESTORE,
            },
        },
    },
};

const MAX_ARTWORK_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    pub available: bool,
    pub title: String,
    pub artist: String,
    pub source: String,
    pub artwork_key: Option<String>,
    pub artwork: Option<String>,
    pub playing: bool,
    pub can_toggle: bool,
    pub can_previous: bool,
    pub can_next: bool,
}

#[derive(Default)]
pub struct MediaController {
    manager: Option<GlobalSystemMediaTransportControlsSessionManager>,
    artwork_key: Option<String>,
    artwork: Option<String>,
}

impl MediaController {
    pub fn state(&mut self, client_artwork_key: Option<&str>) -> Result<MediaState, String> {
        let manager = self.manager()?;
        let Ok(session) = manager.GetCurrentSession() else {
            self.clear_artwork();
            return Ok(MediaState::default());
        };

        let source = session
            .SourceAppUserModelId()
            .map(|value| value.to_string_lossy())
            .unwrap_or_default();
        let properties = session
            .TryGetMediaPropertiesAsync()
            .ok()
            .and_then(|operation| operation.get().ok());
        let title = properties
            .as_ref()
            .and_then(|value| value.Title().ok())
            .map(|value| value.to_string_lossy())
            .unwrap_or_default();
        let artist = properties
            .as_ref()
            .and_then(|value| value.Artist().ok())
            .map(|value| value.to_string_lossy())
            .unwrap_or_default();
        let album = properties
            .as_ref()
            .and_then(|value| value.AlbumTitle().ok())
            .map(|value| value.to_string_lossy())
            .unwrap_or_default();

        let artwork_key = format!("{source}\u{0}{title}\u{0}{artist}\u{0}{album}");
        if self.artwork_key.as_deref() != Some(&artwork_key) {
            self.artwork = properties
                .as_ref()
                .and_then(|value| read_thumbnail(value).ok().flatten());
            self.artwork_key = Some(artwork_key);
        }

        let playback_info = session.GetPlaybackInfo().ok();
        let playing = playback_info
            .as_ref()
            .and_then(|value| value.PlaybackStatus().ok())
            == Some(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing);
        let controls = playback_info.and_then(|value| value.Controls().ok());
        let can_toggle = controls
            .as_ref()
            .map(|value| {
                value.IsPlayPauseToggleEnabled().unwrap_or(false)
                    || value.IsPlayEnabled().unwrap_or(false)
                    || value.IsPauseEnabled().unwrap_or(false)
            })
            .unwrap_or(false);
        let can_next = controls
            .as_ref()
            .and_then(|value| value.IsNextEnabled().ok())
            .unwrap_or(false);
        let can_previous = controls
            .as_ref()
            .and_then(|value| value.IsPreviousEnabled().ok())
            .unwrap_or(false);

        let artwork = if client_artwork_key == self.artwork_key.as_deref() {
            None
        } else {
            self.artwork.clone()
        };

        Ok(MediaState {
            available: true,
            title,
            artist,
            source,
            artwork_key: self.artwork_key.clone(),
            artwork,
            playing,
            can_toggle,
            can_previous,
            can_next,
        })
    }

    pub fn execute(&mut self, action: &str) -> Result<(), String> {
        let manager = self.manager()?;
        let session = manager
            .GetCurrentSession()
            .map_err(|_| "no active Windows media session".to_string())?;

        let performed = match action {
            "previous" => session
                .TrySkipPreviousAsync()
                .and_then(|operation| operation.get()),
            "toggle" => toggle_playback(&session),
            "next" => session
                .TrySkipNextAsync()
                .and_then(|operation| operation.get()),
            _ => return Err(format!("unknown media action: {action}")),
        }
        .map_err(|error| error.to_string())?;

        if performed {
            Ok(())
        } else {
            Err(format!(
                "media action {action} is not supported by the current session"
            ))
        }
    }

    pub fn open_source(&mut self) -> Result<(), String> {
        let manager = self.manager()?;
        let session = manager
            .GetCurrentSession()
            .map_err(|_| "no active Windows media session".to_string())?;
        let source = session
            .SourceAppUserModelId()
            .map(|value| value.to_string_lossy())
            .map_err(|error| error.to_string())?;
        activate_source(&source)
    }

    fn manager(&mut self) -> Result<GlobalSystemMediaTransportControlsSessionManager, String> {
        if self.manager.is_none() {
            self.manager = Some(
                GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                    .and_then(|operation| operation.get())
                    .map_err(|error| error.to_string())?,
            );
        }
        self.manager
            .clone()
            .ok_or_else(|| "Windows media session manager is unavailable".to_string())
    }

    fn clear_artwork(&mut self) {
        self.artwork_key = None;
        self.artwork = None;
    }
}

fn executable_name_from_source(source: &str) -> Option<String> {
    let source = source.trim().trim_matches('"');
    let name = std::path::Path::new(source).file_name()?.to_str()?.trim();
    name.to_ascii_lowercase()
        .ends_with(".exe")
        .then(|| name.to_string())
}

#[cfg(windows)]
fn activate_source(source: &str) -> Result<(), String> {
    let source = source.trim();
    if source.is_empty() {
        return Err("the active media session did not expose a source application".into());
    }

    if let Some(executable) = executable_name_from_source(source) {
        return focus_running_executable(&executable);
    }

    activate_app_user_model_id(source)
}

#[cfg(not(windows))]
fn activate_source(_source: &str) -> Result<(), String> {
    Err("media source activation is only available on Windows".into())
}

#[cfg(windows)]
fn focus_running_executable(executable: &str) -> Result<(), String> {
    let process_ids = process_ids_for_executable(executable)?;
    if process_ids.is_empty() {
        return Err(format!(
            "no running {executable} process has a visible window"
        ));
    }

    let mut search = WindowSearch {
        process_ids: &process_ids,
        window: None,
    };
    unsafe {
        let _ = EnumWindows(
            Some(find_process_window),
            LPARAM((&mut search as *mut WindowSearch<'_>) as isize),
        );
    }
    let window = search
        .window
        .ok_or_else(|| format!("no running {executable} process has a visible window"))?;

    unsafe {
        let _ = ShowWindowAsync(window, SW_RESTORE);
        if !SetForegroundWindow(window).as_bool() {
            return Err(format!("Windows declined to focus {executable}"));
        }
    }
    Ok(())
}

#[cfg(windows)]
struct WindowSearch<'a> {
    process_ids: &'a [u32],
    window: Option<HWND>,
}

#[cfg(windows)]
unsafe extern "system" fn find_process_window(window: HWND, parameter: LPARAM) -> BOOL {
    let search = unsafe { &mut *(parameter.0 as *mut WindowSearch<'_>) };
    if !unsafe { IsWindowVisible(window) }.as_bool() || unsafe { GetWindowTextLengthW(window) } == 0
    {
        return BOOL(1);
    }

    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
    if search.process_ids.contains(&process_id) {
        search.window = Some(window);
        return BOOL(0);
    }
    BOOL(1)
}

#[cfg(windows)]
fn process_ids_for_executable(executable: &str) -> Result<Vec<u32>, String> {
    unsafe {
        let snapshot =
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(|error| error.to_string())?;
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut process_ids = Vec::new();

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let length = entry
                    .szExeFile
                    .iter()
                    .position(|character| *character == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..length]);
                if name.eq_ignore_ascii_case(executable) {
                    process_ids.push(entry.th32ProcessID);
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);
        Ok(process_ids)
    }
}

#[cfg(windows)]
fn activate_app_user_model_id(source: &str) -> Result<(), String> {
    let source_wide: Vec<u16> = source.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|error| error.to_string())?;
        let result = (|| {
            let manager: IApplicationActivationManager =
                CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_LOCAL_SERVER)
                    .map_err(|error| error.to_string())?;
            manager
                .ActivateApplication(PCWSTR(source_wide.as_ptr()), PCWSTR::null(), AO_NOERRORUI)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })();
        CoUninitialize();
        result
    }
}

fn toggle_playback(
    session: &GlobalSystemMediaTransportControlsSession,
) -> windows::core::Result<bool> {
    let info = session.GetPlaybackInfo()?;
    let controls = info.Controls()?;

    if controls.IsPlayPauseToggleEnabled().unwrap_or(false) {
        return session.TryTogglePlayPauseAsync()?.get();
    }

    if info.PlaybackStatus()? == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
        session.TryPauseAsync()?.get()
    } else {
        session.TryPlayAsync()?.get()
    }
}

fn read_thumbnail(
    properties: &GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> windows::core::Result<Option<String>> {
    let reference = properties.Thumbnail()?;
    let stream = reference.OpenReadAsync()?.get()?;
    let size = stream.Size()?;
    if size == 0 || size > MAX_ARTWORK_BYTES || size > u32::MAX as u64 {
        return Ok(None);
    }

    let input = stream.GetInputStreamAt(0)?;
    let reader = DataReader::CreateDataReader(&input)?;
    let loaded = reader.LoadAsync(size as u32)?.get()?;
    if loaded == 0 {
        let _ = reader.Close();
        return Ok(None);
    }

    let mut bytes = vec![0; loaded as usize];
    reader.ReadBytes(&mut bytes)?;
    let _ = reader.Close();
    let reported_type = stream
        .ContentType()
        .map(|value| value.to_string_lossy())
        .unwrap_or_default();
    let content_type = artwork_content_type(&reported_type, &bytes);

    Ok(Some(format!(
        "data:{content_type};base64,{}",
        STANDARD.encode(bytes)
    )))
}

fn artwork_content_type<'a>(reported: &'a str, bytes: &[u8]) -> &'a str {
    if reported.starts_with("image/") {
        return reported;
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        "image/jpeg"
    } else {
        "image/*"
    }
}

#[cfg(test)]
mod tests {
    use super::{artwork_content_type, executable_name_from_source};

    #[test]
    fn keeps_reported_image_types_and_detects_common_fallbacks() {
        assert_eq!(artwork_content_type("image/webp", b"data"), "image/webp");
        assert_eq!(
            artwork_content_type("application/octet-stream", b"\x89PNG\r\n\x1a\nrest"),
            "image/png"
        );
        assert_eq!(
            artwork_content_type("", &[0xff, 0xd8, 0xff, 0x00]),
            "image/jpeg"
        );
    }

    #[test]
    fn recognizes_desktop_media_sources_without_hard_coding_a_player() {
        assert_eq!(
            executable_name_from_source("Spotify.exe"),
            Some("Spotify.exe".into())
        );
        assert_eq!(
            executable_name_from_source(r#"C:\Program Files\Player\player.exe"#),
            Some("player.exe".into())
        );
        assert_eq!(executable_name_from_source("Vendor.Player_123!App"), None);
    }
}

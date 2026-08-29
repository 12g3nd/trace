use base64::{engine::general_purpose::STANDARD, Engine as _};
use windows::{
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionMediaProperties,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Storage::Streams::DataReader,
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
            can_next,
        })
    }

    pub fn execute(&mut self, action: &str) -> Result<(), String> {
        let manager = self.manager()?;
        let session = manager
            .GetCurrentSession()
            .map_err(|_| "no active Windows media session".to_string())?;

        let performed = match action {
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
    use super::artwork_content_type;

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
}

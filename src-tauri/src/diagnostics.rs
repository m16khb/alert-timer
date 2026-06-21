use crate::key_listener::KeyPress;
use alert_timer_core::TimerEvent;
use serde::Serialize;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KeyDiagnostic {
    pub key: String,
    pub foreground_process: Option<String>,
    pub foreground_title: Option<String>,
    pub reset_profile_ids: Vec<String>,
    pub observed_at_ms: u64,
}

pub fn key_diagnostic(
    key_press: &KeyPress,
    events: &[TimerEvent],
    observed_at_ms: u64,
) -> KeyDiagnostic {
    KeyDiagnostic {
        key: key_press.key.clone(),
        foreground_process: key_press
            .application
            .as_ref()
            .map(|application| application.process_name.clone()),
        foreground_title: key_press
            .application
            .as_ref()
            .map(|application| application.window_title.clone()),
        reset_profile_ids: events
            .iter()
            .map(|event| match event {
                TimerEvent::Reset { profile_id } => profile_id.clone(),
            })
            .collect(),
        observed_at_ms,
    }
}

pub fn write_key_diagnostic(
    app: &AppHandle,
    key_press: &KeyPress,
    events: &[TimerEvent],
    observed_at_ms: u64,
) -> Result<(), String> {
    let path = diagnostics_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let diagnostic = key_diagnostic(key_press, events, observed_at_ms);
    let contents = serde_json::to_string_pretty(&diagnostic).map_err(|error| error.to_string())?;
    fs::write(path, contents).map_err(|error| error.to_string())
}

fn diagnostics_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("key-diagnostics.json"))
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use alert_timer_core::{ActiveApplication, TimerEvent};

    use super::*;

    #[test]
    fn key_diagnostic_records_foreground_app_and_reset_profiles() {
        let diagnostic = key_diagnostic(
            &KeyPress {
                key: "]".to_string(),
                application: Some(ActiveApplication {
                    process_name: "MapleStory.exe".to_string(),
                    window_title: "MapleStory".to_string(),
                }),
            },
            &[TimerEvent::Reset {
                profile_id: "janus".to_string(),
            }],
            12_345,
        );

        assert_eq!(diagnostic.key, "]");
        assert_eq!(
            diagnostic.foreground_process.as_deref(),
            Some("MapleStory.exe")
        );
        assert_eq!(diagnostic.foreground_title.as_deref(), Some("MapleStory"));
        assert_eq!(diagnostic.reset_profile_ids, vec!["janus".to_string()]);
        assert_eq!(diagnostic.observed_at_ms, 12_345);
    }
}

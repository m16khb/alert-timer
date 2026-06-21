use crate::models::AppSettings;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

pub fn load(app: &AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };

    let Ok(contents) = fs::read_to_string(path) else {
        return AppSettings::default();
    };

    let missing_app_filter_profile_ids = missing_app_filter_profile_ids(&contents);

    serde_json::from_str::<AppSettings>(&contents)
        .unwrap_or_default()
        .normalized_with_missing_app_filters(&missing_app_filter_profile_ids)
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    settings.validate()?;
    let path = settings_path(app)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let contents = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, contents).map_err(|error| error.to_string())
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join("settings.json"))
        .map_err(|error| error.to_string())
}

fn missing_app_filter_profile_ids(contents: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(contents) else {
        return Vec::new();
    };

    value
        .get("profiles")
        .and_then(|profiles| profiles.as_array())
        .into_iter()
        .flatten()
        .filter_map(|profile| {
            let profile = profile.as_object()?;
            if profile.contains_key("app_filter") {
                return None;
            }
            profile
                .get("id")
                .and_then(|id| id.as_str())
                .map(ToOwned::to_owned)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_app_filter_ids_detects_legacy_profiles_only() {
        let contents = r##"{
          "profiles": [
            {
              "id": "janus",
              "name": "야누스",
              "key": "]",
              "duration_seconds": 120,
              "warning_before_seconds": 5,
              "color": "#ff3344",
              "cycle_key_count": 3,
              "enabled": true
            },
            {
              "id": "all-apps",
              "name": "All Apps",
              "key": "A",
              "app_filter": "",
              "duration_seconds": 120,
              "warning_before_seconds": 5,
              "color": "#20c7a7",
              "cycle_key_count": 1,
              "enabled": true
            }
          ],
          "overlay": {
            "border_thickness_px": 8
          }
        }"##;

        assert_eq!(
            missing_app_filter_profile_ids(contents),
            vec!["janus".to_string()]
        );
    }

    #[test]
    fn legacy_janus_without_app_filter_loads_with_maplestory_filter() {
        let contents = r##"{
          "profiles": [
            {
              "id": "janus",
              "name": "야누스",
              "key": "]",
              "duration_seconds": 120,
              "warning_before_seconds": 5,
              "color": "#ff3344",
              "cycle_key_count": 3,
              "enabled": true
            }
          ],
          "overlay": {
            "border_thickness_px": 8
          }
        }"##;

        let missing_ids = missing_app_filter_profile_ids(contents);
        let settings = serde_json::from_str::<AppSettings>(contents)
            .expect("legacy settings should parse")
            .normalized_with_missing_app_filters(&missing_ids);

        assert_eq!(settings.profiles[0].app_filter, "MapleStory");
    }

    #[test]
    fn explicit_blank_app_filter_stays_blank() {
        let contents = r##"{
          "profiles": [
            {
              "id": "janus",
              "name": "야누스",
              "key": "]",
              "app_filter": "",
              "duration_seconds": 120,
              "warning_before_seconds": 5,
              "color": "#ff3344",
              "cycle_key_count": 3,
              "enabled": true
            }
          ],
          "overlay": {
            "border_thickness_px": 8
          }
        }"##;

        let missing_ids = missing_app_filter_profile_ids(contents);
        let settings = serde_json::from_str::<AppSettings>(contents)
            .expect("settings should parse")
            .normalized_with_missing_app_filters(&missing_ids);

        assert_eq!(settings.profiles[0].app_filter, "");
    }
}

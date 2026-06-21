use alert_timer_core::{AlertPhase, AlertSnapshot, TimerPhase, TimerProfile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub profiles: Vec<SkillProfile>,
    pub overlay: OverlaySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlaySettings {
    pub border_thickness_px: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProfile {
    pub id: String,
    pub name: String,
    pub key: String,
    pub duration_seconds: u64,
    pub warning_before_seconds: u64,
    pub color: String,
    #[serde(alias = "skill_press_count")]
    pub cycle_key_count: u8,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimerSnapshot {
    pub profile_id: String,
    pub name: String,
    pub color: String,
    pub phase: String,
    pub warning_before_ms: u64,
    pub remaining_ms: Option<u64>,
    pub overdue_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlayFramePayload {
    pub active: bool,
    pub color: String,
    pub visible: bool,
    pub intensity: String,
    pub border_thickness_px: u16,
    pub alerts: Vec<OverlayAlertPayload>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlayAlertPayload {
    pub profile_id: String,
    pub name: String,
    pub color: String,
    pub phase: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            profiles: vec![SkillProfile {
                id: "janus".to_string(),
                name: "야누스".to_string(),
                key: "]".to_string(),
                duration_seconds: 120,
                warning_before_seconds: 5,
                color: "#ff3344".to_string(),
                cycle_key_count: 3,
                enabled: true,
            }],
            overlay: OverlaySettings {
                border_thickness_px: 8,
            },
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.overlay.border_thickness_px < 2 || self.overlay.border_thickness_px > 32 {
            return Err("테두리 두께는 2-32px 사이여야 합니다.".to_string());
        }

        for profile in &self.profiles {
            profile.validate()?;
        }

        Ok(())
    }

    pub fn to_timer_profiles(&self) -> Vec<TimerProfile> {
        self.profiles
            .iter()
            .map(SkillProfile::to_timer_profile)
            .collect()
    }
}

impl SkillProfile {
    fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("프로필 ID가 비어 있습니다.".to_string());
        }
        if self.name.trim().is_empty() {
            return Err("프로필 이름이 비어 있습니다.".to_string());
        }
        if self.enabled && self.key.trim().is_empty() {
            return Err(format!("{} 프로필의 키가 비어 있습니다.", self.name));
        }
        if self.duration_seconds < 5 || self.duration_seconds > 3600 {
            return Err(format!("{} 타이머는 5-3600초 사이여야 합니다.", self.name));
        }
        if self.warning_before_seconds >= self.duration_seconds {
            return Err(format!(
                "{} 점멸 시작 시간은 타이머 시간보다 작아야 합니다.",
                self.name
            ));
        }
        if self.cycle_key_count == 0 || self.cycle_key_count > 10 {
            return Err(format!(
                "{} 한 사이클 키 입력 수는 1-10 사이여야 합니다.",
                self.name
            ));
        }
        if !self.color.starts_with('#')
            || self.color.len() != 7
            || !self.color[1..].chars().all(|ch| ch.is_ascii_hexdigit())
        {
            return Err(format!("{} 색상은 #RRGGBB 형식이어야 합니다.", self.name));
        }
        Ok(())
    }

    fn to_timer_profile(&self) -> TimerProfile {
        TimerProfile {
            id: self.id.clone(),
            name: self.name.clone(),
            key: self.key.clone(),
            duration_ms: self.duration_seconds.saturating_mul(1000),
            warning_before_ms: self.warning_before_seconds.saturating_mul(1000),
            color: self.color.clone(),
            cycle_key_count: self.cycle_key_count,
            enabled: self.enabled,
        }
    }
}

pub fn snapshot_from_phase(profile: &TimerProfile, phase: TimerPhase) -> TimerSnapshot {
    match phase {
        TimerPhase::Waiting => TimerSnapshot {
            profile_id: profile.id.clone(),
            name: profile.name.clone(),
            color: profile.color.clone(),
            phase: "waiting".to_string(),
            warning_before_ms: profile.warning_before_ms,
            remaining_ms: None,
            overdue_ms: None,
        },
        TimerPhase::Running { remaining_ms } => TimerSnapshot {
            profile_id: profile.id.clone(),
            name: profile.name.clone(),
            color: profile.color.clone(),
            phase: "running".to_string(),
            warning_before_ms: profile.warning_before_ms,
            remaining_ms: Some(remaining_ms),
            overdue_ms: None,
        },
        TimerPhase::Warning { remaining_ms } => TimerSnapshot {
            profile_id: profile.id.clone(),
            name: profile.name.clone(),
            color: profile.color.clone(),
            phase: "warning".to_string(),
            warning_before_ms: profile.warning_before_ms,
            remaining_ms: Some(remaining_ms),
            overdue_ms: None,
        },
        TimerPhase::Expired { overdue_ms } => TimerSnapshot {
            profile_id: profile.id.clone(),
            name: profile.name.clone(),
            color: profile.color.clone(),
            phase: "expired".to_string(),
            warning_before_ms: profile.warning_before_ms,
            remaining_ms: None,
            overdue_ms: Some(overdue_ms),
        },
    }
}

pub fn alert_payloads(alerts: &[AlertSnapshot]) -> Vec<OverlayAlertPayload> {
    alerts
        .iter()
        .map(|alert| OverlayAlertPayload {
            profile_id: alert.profile_id.clone(),
            name: alert.name.clone(),
            color: alert.color.clone(),
            phase: match alert.phase {
                AlertPhase::Warning => "warning",
                AlertPhase::Expired => "expired",
            }
            .to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with_profile(profile: SkillProfile) -> AppSettings {
        AppSettings {
            profiles: vec![profile],
            overlay: OverlaySettings {
                border_thickness_px: 8,
            },
        }
    }

    fn valid_profile() -> SkillProfile {
        SkillProfile {
            id: "janus".to_string(),
            name: "야누스".to_string(),
            key: "A".to_string(),
            duration_seconds: 120,
            warning_before_seconds: 5,
            color: "#ff3344".to_string(),
            cycle_key_count: 3,
            enabled: true,
        }
    }

    #[test]
    fn default_janus_key_is_closing_bracket() {
        let settings = AppSettings::default();

        assert_eq!(settings.profiles[0].id, "janus");
        assert_eq!(settings.profiles[0].key, "]");
    }

    #[test]
    fn validation_accepts_inclusive_boundary_values() {
        let mut profile = valid_profile();
        profile.duration_seconds = 5;
        profile.warning_before_seconds = 4;
        profile.cycle_key_count = 1;
        profile.color = "#A1b2C3".to_string();

        let mut settings = settings_with_profile(profile);
        settings.overlay.border_thickness_px = 2;
        assert!(settings.validate().is_ok());

        settings.overlay.border_thickness_px = 32;
        settings.profiles[0].duration_seconds = 3600;
        settings.profiles[0].warning_before_seconds = 3599;
        settings.profiles[0].cycle_key_count = 10;
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validation_rejects_values_just_outside_boundaries() {
        let mut settings = settings_with_profile(valid_profile());

        settings.overlay.border_thickness_px = 1;
        assert!(settings.validate().is_err());
        settings.overlay.border_thickness_px = 33;
        assert!(settings.validate().is_err());
        settings.overlay.border_thickness_px = 8;

        settings.profiles[0].duration_seconds = 4;
        assert!(settings.validate().is_err());
        settings.profiles[0].duration_seconds = 3601;
        assert!(settings.validate().is_err());
        settings.profiles[0].duration_seconds = 120;

        settings.profiles[0].warning_before_seconds = 120;
        assert!(settings.validate().is_err());
        settings.profiles[0].warning_before_seconds = 5;

        settings.profiles[0].cycle_key_count = 0;
        assert!(settings.validate().is_err());
        settings.profiles[0].cycle_key_count = 11;
        assert!(settings.validate().is_err());
        settings.profiles[0].cycle_key_count = 3;
    }

    #[test]
    fn validation_allows_blank_key_only_when_profile_is_disabled() {
        let mut profile = valid_profile();
        profile.key = " ".to_string();
        assert!(settings_with_profile(profile.clone()).validate().is_err());

        profile.enabled = false;
        assert!(settings_with_profile(profile).validate().is_ok());
    }

    #[test]
    fn validation_rejects_empty_identity_fields() {
        let mut profile = valid_profile();
        profile.id = " ".to_string();
        assert!(settings_with_profile(profile).validate().is_err());

        let mut profile = valid_profile();
        profile.name = " ".to_string();
        assert!(settings_with_profile(profile).validate().is_err());
    }

    #[test]
    fn validation_rejects_non_hex_colors() {
        let mut profile = valid_profile();
        profile.color = "#zzzzzz".to_string();

        assert!(settings_with_profile(profile).validate().is_err());
    }

    #[test]
    fn timer_profile_conversion_uses_milliseconds() {
        let profile = valid_profile();
        let timer_profile = profile.to_timer_profile();

        assert_eq!(timer_profile.id, "janus");
        assert_eq!(timer_profile.duration_ms, 120_000);
        assert_eq!(timer_profile.warning_before_ms, 5_000);
        assert_eq!(timer_profile.cycle_key_count, 3);
    }

    #[test]
    fn legacy_skill_press_count_settings_load_as_cycle_key_count() {
        let settings: AppSettings = serde_json::from_str(
            r##"{
              "profiles": [
                {
                  "id": "janus",
                  "name": "야누스",
                  "key": "]",
                  "duration_seconds": 120,
                  "warning_before_seconds": 5,
                  "color": "#ff3344",
                  "skill_press_count": 3,
                  "repeat_ignore_window_seconds": 10,
                  "enabled": true
                }
              ],
              "overlay": {
                "border_thickness_px": 8
              }
            }"##,
        )
        .expect("legacy settings should deserialize");

        assert_eq!(settings.profiles[0].cycle_key_count, 3);
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn snapshots_and_alert_payloads_map_phases_for_frontend() {
        let timer_profile = valid_profile().to_timer_profile();

        assert_eq!(
            snapshot_from_phase(&timer_profile, TimerPhase::Waiting).phase,
            "waiting"
        );
        assert_eq!(
            snapshot_from_phase(&timer_profile, TimerPhase::Waiting).warning_before_ms,
            5_000
        );
        assert_eq!(
            snapshot_from_phase(&timer_profile, TimerPhase::Running { remaining_ms: 1 }).phase,
            "running"
        );
        assert_eq!(
            snapshot_from_phase(&timer_profile, TimerPhase::Warning { remaining_ms: 1 }).phase,
            "warning"
        );
        assert_eq!(
            snapshot_from_phase(&timer_profile, TimerPhase::Expired { overdue_ms: 1 }).phase,
            "expired"
        );

        let payloads = alert_payloads(&[
            AlertSnapshot {
                profile_id: "warning".to_string(),
                name: "warning".to_string(),
                color: "#111111".to_string(),
                phase: AlertPhase::Warning,
                remaining_ms: Some(1),
                overdue_ms: None,
            },
            AlertSnapshot {
                profile_id: "expired".to_string(),
                name: "expired".to_string(),
                color: "#222222".to_string(),
                phase: AlertPhase::Expired,
                remaining_ms: None,
                overdue_ms: Some(1),
            },
        ]);

        assert_eq!(payloads[0].phase, "warning");
        assert_eq!(payloads[1].phase, "expired");
    }
}

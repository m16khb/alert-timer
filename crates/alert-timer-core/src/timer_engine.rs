#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerProfile {
    pub id: String,
    pub name: String,
    pub key: String,
    pub duration_ms: u64,
    pub warning_before_ms: u64,
    pub color: String,
    pub skill_press_count: u8,
    pub repeat_ignore_window_ms: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerEvent {
    Reset { profile_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerPhase {
    Waiting,
    Running { remaining_ms: u64 },
    Warning { remaining_ms: u64 },
    Expired { overdue_ms: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertPhase {
    Warning,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertSnapshot {
    pub profile_id: String,
    pub name: String,
    pub color: String,
    pub phase: AlertPhase,
    pub remaining_ms: Option<u64>,
    pub overdue_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverlayIntensity {
    Warning,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayFrame {
    pub color: String,
    pub visible: bool,
    pub intensity: OverlayIntensity,
}

#[derive(Debug, Clone)]
struct TimerRuntime {
    profile: TimerProfile,
    started_at_ms: Option<u64>,
    ignore_until_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct TimerEngine {
    runtimes: Vec<TimerRuntime>,
}

impl TimerEngine {
    pub fn new(profiles: Vec<TimerProfile>) -> Self {
        Self {
            runtimes: profiles
                .into_iter()
                .map(|profile| TimerRuntime {
                    profile,
                    started_at_ms: None,
                    ignore_until_ms: None,
                })
                .collect(),
        }
    }

    pub fn profiles(&self) -> Vec<TimerProfile> {
        self.runtimes
            .iter()
            .map(|runtime| runtime.profile.clone())
            .collect()
    }

    pub fn replace_profiles(&mut self, profiles: Vec<TimerProfile>) {
        self.runtimes = profiles
            .into_iter()
            .map(|profile| TimerRuntime {
                profile,
                started_at_ms: None,
                ignore_until_ms: None,
            })
            .collect();
    }

    pub fn handle_key_press(&mut self, key: &str, now_ms: u64) -> Vec<TimerEvent> {
        let normalized_key = normalize_key(key);
        let mut events = Vec::new();

        for runtime in &mut self.runtimes {
            if !runtime.profile.enabled || normalize_key(&runtime.profile.key) != normalized_key {
                continue;
            }

            if runtime
                .ignore_until_ms
                .is_some_and(|ignore_until_ms| now_ms < ignore_until_ms)
            {
                continue;
            }

            runtime.started_at_ms = Some(now_ms);
            runtime.ignore_until_ms = ignore_until(runtime.profile.skill_press_count, runtime.profile.repeat_ignore_window_ms, now_ms);
            events.push(TimerEvent::Reset {
                profile_id: runtime.profile.id.clone(),
            });
        }

        events
    }

    pub fn phase(&self, profile_id: &str, now_ms: u64) -> Option<TimerPhase> {
        self.runtimes
            .iter()
            .find(|runtime| runtime.profile.id == profile_id)
            .map(|runtime| runtime.phase(now_ms))
    }

    pub fn active_alerts(&self, now_ms: u64) -> Vec<AlertSnapshot> {
        self.runtimes
            .iter()
            .filter_map(|runtime| match runtime.phase(now_ms) {
                TimerPhase::Warning { remaining_ms } => Some(AlertSnapshot {
                    profile_id: runtime.profile.id.clone(),
                    name: runtime.profile.name.clone(),
                    color: runtime.profile.color.clone(),
                    phase: AlertPhase::Warning,
                    remaining_ms: Some(remaining_ms),
                    overdue_ms: None,
                }),
                TimerPhase::Expired { overdue_ms } => Some(AlertSnapshot {
                    profile_id: runtime.profile.id.clone(),
                    name: runtime.profile.name.clone(),
                    color: runtime.profile.color.clone(),
                    phase: AlertPhase::Expired,
                    remaining_ms: None,
                    overdue_ms: Some(overdue_ms),
                }),
                TimerPhase::Waiting | TimerPhase::Running { .. } => None,
            })
            .collect()
    }
}

impl TimerRuntime {
    fn phase(&self, now_ms: u64) -> TimerPhase {
        let Some(started_at_ms) = self.started_at_ms else {
            return TimerPhase::Waiting;
        };

        let elapsed_ms = now_ms.saturating_sub(started_at_ms);
        if elapsed_ms >= self.profile.duration_ms {
            return TimerPhase::Expired {
                overdue_ms: elapsed_ms - self.profile.duration_ms,
            };
        }

        let remaining_ms = self.profile.duration_ms - elapsed_ms;
        if remaining_ms <= self.profile.warning_before_ms {
            TimerPhase::Warning { remaining_ms }
        } else {
            TimerPhase::Running { remaining_ms }
        }
    }
}

pub fn overlay_frame(alerts: &[AlertSnapshot], now_ms: u64) -> Option<OverlayFrame> {
    if alerts.is_empty() {
        return None;
    }

    let intensity = if alerts.iter().any(|alert| alert.phase == AlertPhase::Expired) {
        OverlayIntensity::Expired
    } else {
        OverlayIntensity::Warning
    };
    let color_index = ((now_ms / 400) as usize) % alerts.len();
    let blink_ms = match intensity {
        OverlayIntensity::Warning => 550,
        OverlayIntensity::Expired => 300,
    };

    Some(OverlayFrame {
        color: alerts[color_index].color.clone(),
        visible: (now_ms / blink_ms) % 2 == 0,
        intensity,
    })
}

fn ignore_until(skill_press_count: u8, repeat_ignore_window_ms: u64, now_ms: u64) -> Option<u64> {
    if skill_press_count > 1 && repeat_ignore_window_ms > 0 {
        Some(now_ms.saturating_add(repeat_ignore_window_ms))
    } else {
        None
    }
}

fn normalize_key(key: &str) -> String {
    key.trim().to_ascii_uppercase()
}

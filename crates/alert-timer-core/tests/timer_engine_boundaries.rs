use alert_timer_core::{
    ActiveApplication, AlertPhase, AlertSnapshot, OverlayFrame, OverlayIntensity, TimerEngine,
    TimerEvent, TimerPhase, TimerProfile, overlay_frame,
};

fn janus_profile() -> TimerProfile {
    TimerProfile {
        id: "janus".to_string(),
        name: "야누스".to_string(),
        key: "A".to_string(),
        app_filter: "".to_string(),
        duration_ms: 120_000,
        warning_before_ms: 5_000,
        color: "#ff3344".to_string(),
        cycle_key_count: 3,
        enabled: true,
    }
}

fn profile_with_id(id: &str) -> TimerProfile {
    TimerProfile {
        id: id.to_string(),
        name: id.to_string(),
        key: "A".to_string(),
        app_filter: "".to_string(),
        duration_ms: 10_000,
        warning_before_ms: 2_000,
        color: "#ff3344".to_string(),
        cycle_key_count: 1,
        enabled: true,
    }
}

fn active_application(process_name: &str, window_title: &str) -> ActiveApplication {
    ActiveApplication {
        process_name: process_name.to_string(),
        window_title: window_title.to_string(),
    }
}

#[test]
fn multi_press_skills_group_a_fixed_number_of_keydowns_per_cycle() {
    let mut engine = TimerEngine::new(vec![janus_profile()]);

    assert_eq!(
        engine.handle_key_press("A", 1_000),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(
        engine.handle_key_press("A", 1_100),
        Vec::<TimerEvent>::new()
    );
    assert_eq!(
        engine.handle_key_press("A", 1_200),
        Vec::<TimerEvent>::new()
    );
    assert_eq!(
        engine.handle_key_press("A", 1_300),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(
        engine.phase("janus", 1_300),
        Some(TimerPhase::Running {
            remaining_ms: 120_000
        })
    );
}

#[test]
fn incomplete_cycles_reset_to_zero_after_thirty_seconds() {
    let mut engine = TimerEngine::new(vec![janus_profile()]);

    assert_eq!(
        engine.handle_key_press("A", 1_000),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(
        engine.handle_key_press("A", 30_999),
        Vec::<TimerEvent>::new()
    );
    assert_eq!(
        engine.handle_key_press("A", 31_000),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(
        engine.phase("janus", 31_000),
        Some(TimerPhase::Running {
            remaining_ms: 120_000
        })
    );
}

#[test]
fn first_key_press_resets_timer_and_followup_presses_in_cycle_are_ignored() {
    let mut engine = TimerEngine::new(vec![janus_profile()]);

    assert_eq!(
        engine.handle_key_press("A", 1_000),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );

    assert_eq!(
        engine.phase("janus", 1_000),
        Some(TimerPhase::Running {
            remaining_ms: 120_000
        })
    );
    assert_eq!(
        engine.handle_key_press("A", 2_000),
        Vec::<TimerEvent>::new()
    );
    assert_eq!(
        engine.phase("janus", 2_000),
        Some(TimerPhase::Running {
            remaining_ms: 119_000
        })
    );
    assert_eq!(
        engine.handle_key_press("A", 10_999),
        Vec::<TimerEvent>::new()
    );

    assert_eq!(
        engine.handle_key_press("A", 11_001),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(
        engine.phase("janus", 11_001),
        Some(TimerPhase::Running {
            remaining_ms: 120_000
        })
    );
}

#[test]
fn warning_phase_resets_on_first_matching_key_even_inside_cycle_window() {
    let mut profile = janus_profile();
    profile.duration_ms = 20_000;
    profile.warning_before_ms = 5_000;
    let mut engine = TimerEngine::new(vec![profile]);

    assert_eq!(
        engine.handle_key_press("A", 1_000),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(
        engine.phase("janus", 16_000),
        Some(TimerPhase::Warning {
            remaining_ms: 5_000
        })
    );
    assert_eq!(
        engine.handle_key_press("A", 16_000),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(
        engine.phase("janus", 16_000),
        Some(TimerPhase::Running {
            remaining_ms: 20_000
        })
    );
    assert_eq!(
        engine.handle_key_press("A", 16_100),
        Vec::<TimerEvent>::new()
    );
}

#[test]
fn expired_phase_resets_on_first_matching_key_even_inside_cycle_window() {
    let mut profile = janus_profile();
    profile.duration_ms = 10_000;
    profile.warning_before_ms = 2_000;
    let mut engine = TimerEngine::new(vec![profile]);

    assert_eq!(
        engine.handle_key_press("A", 1_000),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(
        engine.phase("janus", 12_000),
        Some(TimerPhase::Expired { overdue_ms: 1_000 })
    );
    assert_eq!(
        engine.handle_key_press("A", 12_000),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(
        engine.phase("janus", 12_000),
        Some(TimerPhase::Running {
            remaining_ms: 10_000
        })
    );
}

#[test]
fn non_matching_keys_do_not_advance_the_current_cycle() {
    let mut engine = TimerEngine::new(vec![janus_profile()]);

    assert_eq!(
        engine.handle_key_press("A", 0),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
    assert_eq!(engine.handle_key_press("B", 1), Vec::<TimerEvent>::new());
    assert_eq!(engine.handle_key_press("A", 2), Vec::<TimerEvent>::new());
    assert_eq!(engine.handle_key_press("A", 3), Vec::<TimerEvent>::new());
    assert_eq!(
        engine.handle_key_press("A", 4),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
}

#[test]
fn key_matching_is_case_insensitive_and_trims_input() {
    let mut profile = janus_profile();
    profile.key = " f12 ".to_string();
    let mut engine = TimerEngine::new(vec![profile]);

    assert_eq!(
        engine.handle_key_press("F12", 0),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
}

#[test]
fn app_filter_resets_only_for_matching_foreground_application() {
    let mut profile = janus_profile();
    profile.app_filter = "MapleStory".to_string();
    let mut engine = TimerEngine::new(vec![profile]);

    assert_eq!(
        engine.handle_key_press_with_app(
            "A",
            Some(&active_application("notepad.exe", "Untitled - Notepad")),
            0
        ),
        Vec::<TimerEvent>::new()
    );
    assert_eq!(engine.phase("janus", 0), Some(TimerPhase::Waiting));

    assert_eq!(
        engine.handle_key_press_with_app(
            "A",
            Some(&active_application("MapleStory.exe", "MapleStory")),
            1
        ),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
}

#[test]
fn app_filter_can_match_window_title_when_process_name_differs() {
    let mut profile = janus_profile();
    profile.app_filter = "maplestory".to_string();
    let mut engine = TimerEngine::new(vec![profile]);

    assert_eq!(
        engine.handle_key_press_with_app(
            "A",
            Some(&active_application("GameLauncher.exe", "MapleStory")),
            0
        ),
        vec![TimerEvent::Reset {
            profile_id: "janus".to_string()
        }]
    );
}

#[test]
fn disabled_profiles_and_unknown_keys_do_not_reset() {
    let mut disabled = janus_profile();
    disabled.enabled = false;
    let mut engine = TimerEngine::new(vec![disabled]);

    assert_eq!(engine.handle_key_press("A", 0), Vec::<TimerEvent>::new());
    assert_eq!(engine.phase("janus", 10_000), Some(TimerPhase::Waiting));
    assert_eq!(engine.phase("missing", 10_000), None);
    assert!(engine.active_alerts(10_000).is_empty());
}

#[test]
fn single_press_profiles_reset_on_every_matching_keydown() {
    let mut profile = janus_profile();
    profile.cycle_key_count = 1;
    let mut engine = TimerEngine::new(vec![profile]);

    assert_eq!(engine.handle_key_press("A", 0).len(), 1);
    assert_eq!(engine.handle_key_press("A", 1).len(), 1);
    assert_eq!(
        engine.phase("janus", 1),
        Some(TimerPhase::Running {
            remaining_ms: 120_000
        })
    );
}

#[test]
fn phase_boundaries_switch_at_warning_and_expiration_edges() {
    let mut engine = TimerEngine::new(vec![profile_with_id("edge")]);
    engine.handle_key_press("A", 1_000);

    assert_eq!(
        engine.phase("edge", 1_000),
        Some(TimerPhase::Running {
            remaining_ms: 10_000
        })
    );
    assert_eq!(
        engine.phase("edge", 8_999),
        Some(TimerPhase::Running {
            remaining_ms: 2_001
        })
    );
    assert_eq!(
        engine.phase("edge", 9_000),
        Some(TimerPhase::Warning {
            remaining_ms: 2_000
        })
    );
    assert_eq!(
        engine.phase("edge", 10_999),
        Some(TimerPhase::Warning { remaining_ms: 1 })
    );
    assert_eq!(
        engine.phase("edge", 11_000),
        Some(TimerPhase::Expired { overdue_ms: 0 })
    );
    assert_eq!(
        engine.phase("edge", 11_001),
        Some(TimerPhase::Expired { overdue_ms: 1 })
    );
}

#[test]
fn phase_queries_before_start_time_do_not_underflow() {
    let mut engine = TimerEngine::new(vec![profile_with_id("clock")]);
    engine.handle_key_press("A", 5_000);

    assert_eq!(
        engine.phase("clock", 4_000),
        Some(TimerPhase::Running {
            remaining_ms: 10_000
        })
    );
}

#[test]
fn replace_profiles_resets_runtime_state_and_keeps_new_profiles() {
    let mut engine = TimerEngine::new(vec![profile_with_id("old")]);
    engine.handle_key_press("A", 0);
    assert_eq!(
        engine.phase("old", 9_000),
        Some(TimerPhase::Warning {
            remaining_ms: 1_000
        })
    );

    let mut replacement = profile_with_id("new");
    replacement.key = "B".to_string();
    engine.replace_profiles(vec![replacement.clone()]);

    assert_eq!(engine.profiles(), vec![replacement]);
    assert_eq!(engine.phase("old", 9_000), None);
    assert_eq!(engine.phase("new", 9_000), Some(TimerPhase::Waiting));
    assert_eq!(
        engine.handle_key_press("A", 9_000),
        Vec::<TimerEvent>::new()
    );
    assert_eq!(engine.handle_key_press("B", 9_000).len(), 1);
}

#[test]
fn warning_and_expired_alerts_are_reported_for_parallel_profiles() {
    let mut erda = janus_profile();
    erda.id = "erda".to_string();
    erda.name = "에르다 샤워".to_string();
    erda.key = "S".to_string();
    erda.duration_ms = 60_000;
    erda.warning_before_ms = 3_000;
    erda.color = "#2f80ff".to_string();
    erda.cycle_key_count = 1;

    let mut engine = TimerEngine::new(vec![janus_profile(), erda]);
    engine.handle_key_press("A", 0);
    engine.handle_key_press("S", 0);

    assert_eq!(
        engine.phase("janus", 115_000),
        Some(TimerPhase::Warning {
            remaining_ms: 5_000
        })
    );
    assert_eq!(
        engine.phase("erda", 115_000),
        Some(TimerPhase::Expired { overdue_ms: 55_000 })
    );

    let alerts = engine.active_alerts(115_000);
    assert_eq!(alerts.len(), 2);
    assert_eq!(alerts[0].profile_id, "janus");
    assert_eq!(alerts[0].color, "#ff3344");
    assert_eq!(alerts[0].phase, AlertPhase::Warning);
    assert_eq!(alerts[1].profile_id, "erda");
    assert_eq!(alerts[1].color, "#2f80ff");
    assert_eq!(alerts[1].phase, AlertPhase::Expired);
}

#[test]
fn overlay_frame_cycles_alert_colors_and_uses_expired_blink_speed() {
    let alerts = vec![
        AlertSnapshot {
            profile_id: "janus".to_string(),
            name: "야누스".to_string(),
            color: "#ff3344".to_string(),
            phase: AlertPhase::Warning,
            remaining_ms: Some(2_000),
            overdue_ms: None,
        },
        AlertSnapshot {
            profile_id: "erda".to_string(),
            name: "에르다 샤워".to_string(),
            color: "#2f80ff".to_string(),
            phase: AlertPhase::Expired,
            remaining_ms: None,
            overdue_ms: Some(1_000),
        },
    ];

    assert_eq!(
        overlay_frame(&alerts, 0),
        Some(OverlayFrame {
            color: "#ff3344".to_string(),
            visible: true,
            intensity: OverlayIntensity::Expired,
        })
    );
    assert_eq!(
        overlay_frame(&alerts, 450),
        Some(OverlayFrame {
            color: "#2f80ff".to_string(),
            visible: false,
            intensity: OverlayIntensity::Expired,
        })
    );
}

#[test]
fn overlay_frame_returns_none_without_alerts() {
    assert_eq!(overlay_frame(&[], 0), None);
}

#[test]
fn overlay_frame_uses_warning_blink_speed_without_expired_alerts() {
    let alerts = vec![AlertSnapshot {
        profile_id: "janus".to_string(),
        name: "야누스".to_string(),
        color: "#ff3344".to_string(),
        phase: AlertPhase::Warning,
        remaining_ms: Some(1_000),
        overdue_ms: None,
    }];

    assert_eq!(
        overlay_frame(&alerts, 549),
        Some(OverlayFrame {
            color: "#ff3344".to_string(),
            visible: true,
            intensity: OverlayIntensity::Warning,
        })
    );
    assert_eq!(
        overlay_frame(&alerts, 550),
        Some(OverlayFrame {
            color: "#ff3344".to_string(),
            visible: false,
            intensity: OverlayIntensity::Warning,
        })
    );
}

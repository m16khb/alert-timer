mod key_listener;
mod models;
mod overlay;
mod privilege;
mod settings_store;
mod tray;

use alert_timer_core::{OverlayIntensity, TimerEngine, overlay_frame};
use models::{
    AppSettings, OverlayFramePayload, TimerSnapshot, alert_payloads, snapshot_from_phase,
};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

struct AppState {
    engine: Mutex<TimerEngine>,
    settings: Mutex<AppSettings>,
    started_at: Instant,
}

impl AppState {
    fn new(settings: AppSettings) -> Self {
        Self {
            engine: Mutex::new(TimerEngine::new(settings.to_timer_profiles())),
            settings: Mutex::new(settings),
            started_at: Instant::now(),
        }
    }

    fn now_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }

    fn replace_settings(&self, settings: AppSettings) {
        self.engine
            .lock()
            .replace_profiles(settings.to_timer_profiles());
        *self.settings.lock() = settings;
    }

    fn snapshots(&self, now_ms: u64) -> Vec<TimerSnapshot> {
        let engine = self.engine.lock();
        engine
            .profiles()
            .into_iter()
            .filter_map(|profile| {
                engine
                    .phase(&profile.id, now_ms)
                    .map(|phase| snapshot_from_phase(&profile, phase))
            })
            .collect()
    }
}

#[tauri::command]
fn get_settings(state: State<'_, Arc<AppState>>) -> AppSettings {
    state.settings.lock().clone()
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    settings.validate()?;
    settings_store::save(&app, &settings)?;
    state.replace_settings(settings.clone());
    publish_state(&app, &state);
    Ok(settings)
}

#[tauri::command]
fn get_timer_snapshots(state: State<'_, Arc<AppState>>) -> Vec<TimerSnapshot> {
    state.snapshots(state.now_ms())
}

#[tauri::command]
fn get_privilege_status() -> privilege::PrivilegeStatus {
    privilege::status()
}

#[tauri::command]
fn relaunch_as_admin() -> Result<(), String> {
    privilege::relaunch_as_admin()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let settings = settings_store::load(app.handle());
            let state = Arc::new(AppState::new(settings));
            app.manage(state.clone());

            overlay::configure(app.handle())?;
            tray::build(app.handle())?;
            start_runtime_loop(app.handle().clone(), state);

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" || window.label() == "mini" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_timer_snapshots,
            get_privilege_status,
            relaunch_as_admin
        ])
        .run(tauri::generate_context!())
        .expect("failed to run AlertTimer");
}

fn start_runtime_loop(app: AppHandle, state: Arc<AppState>) {
    let (sender, receiver) = mpsc::channel::<key_listener::KeyPress>();
    if let Err(error) = key_listener::start(sender) {
        eprintln!("key listener unavailable: {error}");
    }

    thread::Builder::new()
        .name("alert-timer-runtime".to_string())
        .spawn(move || {
            loop {
                while let Ok(key_press) = receiver.try_recv() {
                    let now_ms = state.now_ms();
                    let mut engine = state.engine.lock();
                    let events = engine.handle_key_press_with_app(
                        &key_press.key,
                        key_press.application.as_ref(),
                        now_ms,
                    );
                    drop(engine);

                    if !events.is_empty() {
                        publish_state(&app, &state);
                    }
                }

                publish_state(&app, &state);
                thread::sleep(Duration::from_millis(150));
            }
        })
        .expect("failed to start runtime loop");
}

fn publish_state(app: &AppHandle, state: &Arc<AppState>) {
    let now_ms = state.now_ms();
    let settings = state.settings.lock().clone();
    let engine = state.engine.lock();
    let alerts = engine.active_alerts(now_ms);
    let frame = overlay_frame(&alerts, now_ms);
    let snapshots: Vec<TimerSnapshot> = engine
        .profiles()
        .into_iter()
        .filter_map(|profile| {
            engine
                .phase(&profile.id, now_ms)
                .map(|phase| snapshot_from_phase(&profile, phase))
        })
        .collect();
    drop(engine);

    let _ = app.emit("timer://snapshot", snapshots);

    let payload = frame.map(|frame| OverlayFramePayload {
        active: true,
        color: frame.color,
        visible: frame.visible,
        intensity: match frame.intensity {
            OverlayIntensity::Warning => "warning",
            OverlayIntensity::Expired => "expired",
        }
        .to_string(),
        border_thickness_px: settings.overlay.border_thickness_px,
        alerts: alert_payloads(&alerts),
    });

    overlay::publish(app, payload);
}

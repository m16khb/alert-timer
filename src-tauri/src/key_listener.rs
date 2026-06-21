use std::sync::{Mutex, OnceLock, mpsc::Sender};

static KEY_SENDER: OnceLock<Mutex<Sender<String>>> = OnceLock::new();

#[cfg(target_os = "windows")]
pub fn start(sender: Sender<String>) -> Result<(), String> {
    let _ = KEY_SENDER.set(Mutex::new(sender));

    std::thread::Builder::new()
        .name("alert-timer-key-listener".to_string())
        .spawn(move || {
            windows_impl::poll_loop();
        })
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn start(_sender: Sender<String>) -> Result<(), String> {
    Err("전역 키 감지는 Windows에서만 지원됩니다.".to_string())
}

fn publish_key(key: String) {
    if let Some(sender) = KEY_SENDER.get() {
        if let Ok(sender) = sender.lock() {
            let _ = sender.send(key);
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::publish_key;
    use std::collections::HashSet;
    use std::thread;
    use std::time::Duration;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

    const POLL_INTERVAL: Duration = Duration::from_millis(16);

    pub fn poll_loop() {
        let keys = supported_keys();
        let mut tracker = PressedKeyTracker::default();

        loop {
            for key in keys {
                if tracker.update(key.vk_code, is_key_down(key.vk_code)) {
                    publish_key(key.name.to_string());
                }
            }

            thread::sleep(POLL_INTERVAL);
        }
    }

    fn is_key_down(vk_code: u32) -> bool {
        unsafe { GetAsyncKeyState(vk_code as i32) & 0x8000u16 as i16 != 0 }
    }

    #[derive(Debug, Default)]
    struct PressedKeyTracker {
        pressed: HashSet<u32>,
    }

    impl PressedKeyTracker {
        fn update(&mut self, vk_code: u32, is_down: bool) -> bool {
            if is_down {
                return self.pressed.insert(vk_code);
            }

            self.pressed.remove(&vk_code);
            false
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct KeyDefinition {
        vk_code: u32,
        name: &'static str,
    }

    macro_rules! key {
        ($vk_code:expr, $name:expr) => {
            KeyDefinition {
                vk_code: $vk_code,
                name: $name,
            }
        };
    }

    fn supported_keys() -> &'static [KeyDefinition] {
        &[
            key!(0x30, "0"),
            key!(0x31, "1"),
            key!(0x32, "2"),
            key!(0x33, "3"),
            key!(0x34, "4"),
            key!(0x35, "5"),
            key!(0x36, "6"),
            key!(0x37, "7"),
            key!(0x38, "8"),
            key!(0x39, "9"),
            key!(0x41, "A"),
            key!(0x42, "B"),
            key!(0x43, "C"),
            key!(0x44, "D"),
            key!(0x45, "E"),
            key!(0x46, "F"),
            key!(0x47, "G"),
            key!(0x48, "H"),
            key!(0x49, "I"),
            key!(0x4A, "J"),
            key!(0x4B, "K"),
            key!(0x4C, "L"),
            key!(0x4D, "M"),
            key!(0x4E, "N"),
            key!(0x4F, "O"),
            key!(0x50, "P"),
            key!(0x51, "Q"),
            key!(0x52, "R"),
            key!(0x53, "S"),
            key!(0x54, "T"),
            key!(0x55, "U"),
            key!(0x56, "V"),
            key!(0x57, "W"),
            key!(0x58, "X"),
            key!(0x59, "Y"),
            key!(0x5A, "Z"),
            key!(0x70, "F1"),
            key!(0x71, "F2"),
            key!(0x72, "F3"),
            key!(0x73, "F4"),
            key!(0x74, "F5"),
            key!(0x75, "F6"),
            key!(0x76, "F7"),
            key!(0x77, "F8"),
            key!(0x78, "F9"),
            key!(0x79, "F10"),
            key!(0x7A, "F11"),
            key!(0x7B, "F12"),
            key!(0x20, "Space"),
            key!(0x09, "Tab"),
            key!(0x0D, "Enter"),
            key!(0x1B, "Escape"),
            key!(0x21, "PageUp"),
            key!(0x22, "PageDown"),
            key!(0x23, "End"),
            key!(0x24, "Home"),
            key!(0x25, "ArrowLeft"),
            key!(0x26, "ArrowUp"),
            key!(0x27, "ArrowRight"),
            key!(0x28, "ArrowDown"),
            key!(0x2D, "Insert"),
            key!(0x2E, "Delete"),
            key!(0xDD, "]"),
        ]
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn maps_closing_bracket_key() {
            assert!(
                supported_keys()
                    .iter()
                    .any(|key| key.vk_code == 0xDD && key.name == "]")
            );
        }

        #[test]
        fn key_repeat_is_suppressed_until_keyup() {
            let mut tracker = PressedKeyTracker::default();

            assert!(tracker.update(0xDD, true));
            assert!(!tracker.update(0xDD, true));
            assert!(!tracker.update(0xDD, false));
            assert!(tracker.update(0xDD, true));
        }
    }
}

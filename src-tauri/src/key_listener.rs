use std::sync::{
    Mutex, OnceLock,
    mpsc::{self, Sender},
};
use std::time::Duration;

static KEY_SENDER: OnceLock<Mutex<Sender<String>>> = OnceLock::new();

#[cfg(target_os = "windows")]
pub fn start(sender: Sender<String>) -> Result<(), String> {
    let _ = KEY_SENDER.set(Mutex::new(sender));
    let (ready_sender, ready_receiver) = mpsc::sync_channel(1);

    std::thread::Builder::new()
        .name("alert-timer-key-listener".to_string())
        .spawn(move || {
            windows_impl::message_loop(ready_sender);
        })
        .map_err(|error| error.to_string())?;

    ready_receiver
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| "전역 키 감지 초기화 시간이 초과되었습니다.".to_string())?
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
    use std::sync::mpsc::SyncSender;
    use std::sync::{Mutex, OnceLock};
    use windows_sys::Win32::Foundation::{GetLastError, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, MSG, SetWindowsHookExW,
        TranslateMessage, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    static PRESSED_KEYS: OnceLock<Mutex<PressedKeyTracker>> = OnceLock::new();

    pub fn message_loop(ready_sender: SyncSender<Result<(), String>>) {
        let module_handle = unsafe { GetModuleHandleW(std::ptr::null()) };
        let hook =
            unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module_handle, 0) };
        if hook.is_null() {
            let error_code = unsafe { GetLastError() };
            let _ = ready_sender.send(Err(format!(
                "전역 키 감지 hook 등록에 실패했습니다. Windows error code: {error_code}"
            )));
            return;
        }

        let _ = ready_sender.send(Ok(()));

        let mut message = MSG::default();
        while unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) } > 0 {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    unsafe extern "system" fn keyboard_proc(
        code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if code >= 0 && (w_param as u32 == WM_KEYDOWN || w_param as u32 == WM_SYSKEYDOWN) {
            let keyboard = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };
            if is_fresh_keydown(keyboard.vkCode).is_some_and(|fresh| fresh) {
                if let Some(key) = key_name_from_keyboard(keyboard.vkCode, keyboard.scanCode) {
                    publish_key(key);
                }
            }
        } else if code >= 0 && (w_param as u32 == WM_KEYUP || w_param as u32 == WM_SYSKEYUP) {
            let keyboard = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };
            if let Some(tracker) = PRESSED_KEYS.get() {
                if let Ok(mut tracker) = tracker.lock() {
                    tracker.mark_up(keyboard.vkCode);
                }
            }
        }

        unsafe { CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param) }
    }

    fn is_fresh_keydown(vk_code: u32) -> Option<bool> {
        let tracker = PRESSED_KEYS.get_or_init(|| Mutex::new(PressedKeyTracker::default()));
        tracker
            .lock()
            .ok()
            .map(|mut tracker| tracker.mark_down(vk_code))
    }

    #[derive(Debug, Default)]
    struct PressedKeyTracker {
        pressed: HashSet<u32>,
    }

    impl PressedKeyTracker {
        fn mark_down(&mut self, vk_code: u32) -> bool {
            self.pressed.insert(vk_code)
        }

        fn mark_up(&mut self, vk_code: u32) {
            self.pressed.remove(&vk_code);
        }
    }

    fn key_name_from_keyboard(vk_code: u32, scan_code: u32) -> Option<String> {
        key_name_from_vk(vk_code).or_else(|| key_name_from_scan_code(scan_code))
    }

    fn key_name_from_vk(vk_code: u32) -> Option<String> {
        match vk_code {
            0x30..=0x39 | 0x41..=0x5A => char::from_u32(vk_code).map(|ch| ch.to_string()),
            0x70..=0x87 => Some(format!("F{}", vk_code - 0x6F)),
            0x21 => Some("PageUp".to_string()),
            0x22 => Some("PageDown".to_string()),
            0x23 => Some("End".to_string()),
            0x24 => Some("Home".to_string()),
            0x25 => Some("ArrowLeft".to_string()),
            0x26 => Some("ArrowUp".to_string()),
            0x27 => Some("ArrowRight".to_string()),
            0x28 => Some("ArrowDown".to_string()),
            0x2D => Some("Insert".to_string()),
            0x2E => Some("Delete".to_string()),
            0x20 => Some("Space".to_string()),
            0x09 => Some("Tab".to_string()),
            0x0D => Some("Enter".to_string()),
            0x1B => Some("Escape".to_string()),
            0xDD => Some("]".to_string()),
            _ => None,
        }
    }

    fn key_name_from_scan_code(scan_code: u32) -> Option<String> {
        match scan_code {
            0x1B => Some("]".to_string()),
            _ => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn maps_closing_bracket_key() {
            assert_eq!(key_name_from_vk(0xDD), Some("]".to_string()));
        }

        #[test]
        fn maps_closing_bracket_scan_code_when_vk_is_unknown() {
            assert_eq!(key_name_from_keyboard(0, 0x1B), Some("]".to_string()));
        }

        #[test]
        fn key_repeat_is_suppressed_until_keyup() {
            let mut tracker = PressedKeyTracker::default();

            assert!(tracker.mark_down(0xDD));
            assert!(!tracker.mark_down(0xDD));
            tracker.mark_up(0xDD);
            assert!(tracker.mark_down(0xDD));
        }
    }
}

use std::sync::{mpsc::Sender, Mutex, OnceLock};

static KEY_SENDER: OnceLock<Mutex<Sender<String>>> = OnceLock::new();

#[cfg(target_os = "windows")]
pub fn start(sender: Sender<String>) -> Result<(), String> {
    let _ = KEY_SENDER.set(Mutex::new(sender));

    std::thread::Builder::new()
        .name("alert-timer-key-listener".to_string())
        .spawn(move || {
            windows_impl::message_loop();
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
    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    };

    pub fn message_loop() {
        let hook = unsafe {
            SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), std::ptr::null_mut(), 0)
        };
        if hook.is_null() {
            return;
        }

        let mut message = MSG::default();
        while unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) } > 0 {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    unsafe extern "system" fn keyboard_proc(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        if code >= 0 && (w_param as u32 == WM_KEYDOWN || w_param as u32 == WM_SYSKEYDOWN) {
            let keyboard = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };
            if let Some(key) = key_name_from_vk(keyboard.vkCode) {
                publish_key(key);
            }
        }

        unsafe { CallNextHookEx(std::ptr::null_mut(), code, w_param, l_param) }
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn maps_closing_bracket_key() {
            assert_eq!(key_name_from_vk(0xDD), Some("]".to_string()));
        }
    }
}

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PrivilegeStatus {
    pub is_elevated: bool,
    pub can_relaunch_as_admin: bool,
}

pub fn status() -> PrivilegeStatus {
    PrivilegeStatus {
        is_elevated: is_elevated(),
        can_relaunch_as_admin: cfg!(target_os = "windows") && !is_elevated(),
    }
}

#[cfg(target_os = "windows")]
pub fn relaunch_as_admin() -> Result<(), String> {
    windows_impl::relaunch_as_admin()
}

#[cfg(not(target_os = "windows"))]
pub fn relaunch_as_admin() -> Result<(), String> {
    Err("관리자 권한 재시작은 Windows에서만 지원됩니다.".to_string())
}

#[cfg(target_os = "windows")]
fn is_elevated() -> bool {
    windows_impl::is_elevated()
}

#[cfg(not(target_os = "windows"))]
fn is_elevated() -> bool {
    false
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use std::ffi::OsStr;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null;
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    pub fn is_elevated() -> bool {
        let mut token = std::ptr::null_mut();
        let opened = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) };
        if opened == 0 {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut returned_size = 0;
        let ok = unsafe {
            GetTokenInformation(
                token,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size_of::<TOKEN_ELEVATION>() as u32,
                &mut returned_size,
            )
        };
        unsafe {
            CloseHandle(token);
        }

        ok != 0 && elevation.TokenIsElevated != 0
    }

    pub fn relaunch_as_admin() -> Result<(), String> {
        let exe = std::env::current_exe().map_err(|error| error.to_string())?;
        let operation = wide("runas");
        let file = wide(exe.as_os_str());

        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                operation.as_ptr(),
                file.as_ptr(),
                null(),
                null(),
                SW_SHOWNORMAL,
            )
        };

        if result as isize <= 32 {
            return Err(format!(
                "관리자 권한 재시작 요청에 실패했습니다. Windows error code: {}",
                unsafe { GetLastError() }
            ));
        }

        std::process::exit(0);
    }

    fn wide(value: impl AsRef<OsStr>) -> Vec<u16> {
        value
            .as_ref()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn wide_strings_are_null_terminated() {
            let value = wide("runas");

            assert_eq!(value.last(), Some(&0));
        }
    }
}

//! Windows registry management

use crate::{Result, SmartFreezeError};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_SET_VALUE, KEY_WRITE, REG_SZ,
};

const STARTUP_KEY_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const APP_NAME: &str = "SmartFreeze";

/// Windows registry manager for startup entries
pub struct WindowsRegistry;

impl WindowsRegistry {
    pub fn new() -> Self {
        Self
    }

    /// Convert string to wide (UTF-16) string for Windows API
    fn to_wide_string(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Install SmartFreeze to Windows startup
    pub fn install_startup(&self, exe_path: &str) -> Result<()> {
        unsafe {
            let key_path = Self::to_wide_string(STARTUP_KEY_PATH);
            let mut hkey: HKEY = std::ptr::null_mut();

            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                key_path.as_ptr(),
                0,
                KEY_WRITE,
                &mut hkey,
            );

            if result != 0 {
                return Err(SmartFreezeError::Registry(format!(
                    "Failed to open registry key: error code {}",
                    result
                )));
            }

            let app_name = Self::to_wide_string(APP_NAME);
            let daemon_cmd = format!("\"{}\" --daemon", exe_path);
            let value = Self::to_wide_string(&daemon_cmd);

            let result = RegSetValueExW(
                hkey,
                app_name.as_ptr(),
                0,
                REG_SZ,
                value.as_ptr() as *const u8,
                (value.len() * 2) as u32,
            );

            RegCloseKey(hkey);

            if result != 0 {
                Err(SmartFreezeError::Registry(format!(
                    "Failed to set registry value: error code {}",
                    result
                )))
            } else {
                Ok(())
            }
        }
    }

    /// Uninstall SmartFreeze from Windows startup
    pub fn uninstall_startup(&self) -> Result<()> {
        unsafe {
            let key_path = Self::to_wide_string(STARTUP_KEY_PATH);
            let mut hkey: HKEY = std::ptr::null_mut();

            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                key_path.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            );

            if result != 0 {
                return Err(SmartFreezeError::Registry(format!(
                    "Failed to open registry key: error code {}",
                    result
                )));
            }

            let app_name = Self::to_wide_string(APP_NAME);
            let result = RegDeleteValueW(hkey, app_name.as_ptr());

            RegCloseKey(hkey);

            if result != 0 {
                // Error code 2 means "file not found" which is OK (already uninstalled)
                if result == 2 {
                    Ok(())
                } else {
                    Err(SmartFreezeError::Registry(format!(
                        "Failed to delete registry value: error code {}",
                        result
                    )))
                }
            } else {
                Ok(())
            }
        }
    }

    /// Check if SmartFreeze is installed in startup
    pub fn is_installed(&self) -> bool {
        unsafe {
            let key_path = Self::to_wide_string(STARTUP_KEY_PATH);
            let mut hkey: HKEY = std::ptr::null_mut();

            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                key_path.as_ptr(),
                0,
                KEY_SET_VALUE,
                &mut hkey,
            );

            if result != 0 {
                return false;
            }

            // Try to query the value
            let app_name = Self::to_wide_string(APP_NAME);
            let mut buffer: [u16; 260] = [0; 260];
            let mut buffer_size = (buffer.len() * 2) as u32;

            let result = windows_sys::Win32::System::Registry::RegQueryValueExW(
                hkey,
                app_name.as_ptr(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut u8,
                &mut buffer_size,
            );

            RegCloseKey(hkey);

            result == 0
        }
    }
}

impl Default for WindowsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = WindowsRegistry::new();
        assert!(std::mem::size_of_val(&registry) == 0); // Zero-sized type
    }

    #[test]
    fn test_to_wide_string() {
        let wide = WindowsRegistry::to_wide_string("test");
        assert_eq!(wide.len(), 5); // "test" + null terminator
        assert_eq!(wide[4], 0); // Null terminator
    }

    // Note: Actual registry operations are tested in integration tests
    // to avoid modifying the system during unit tests
}

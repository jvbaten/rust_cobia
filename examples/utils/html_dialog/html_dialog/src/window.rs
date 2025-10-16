#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

/// Represents a native window in the operating system.
///
/// This struct provides a cross-platform abstraction over different window handle types.
/// Currently implemented for Windows with the `HWND` type.
///
/// # Platform-specific details
///
/// - **Windows**: Uses `HWND` from the Windows API as the handle type
///
pub struct Window {
    /// Native window handle.
    ///
    /// On Windows, this is a `HWND` from the Windows API.
    #[cfg(target_os = "windows")]
    pub handle: HWND,
}

impl From<*mut core::ffi::c_void> for Window {
    #[cfg(target_os = "windows")]
    fn from(hwnd: *mut core::ffi::c_void) -> Self {
        Window { handle: HWND(hwnd) }
    }
}

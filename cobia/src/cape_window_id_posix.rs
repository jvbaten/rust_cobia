use crate::C;

/// CapeWindowId represents a window
///
/// X11, Wayland, and AppKit are supported.
#[derive(Clone)]
pub enum CapeWindowId {
	/// No known or unsupported window type
	CapeWindow_None,
	/// X11 window
	CapeWindow_X11 {
		/// Display string
		display_string: String,
		/// Window ID
		xid: u64,
	},
	/// Wayland window
	CapeWindow_Wayland {
		/// Wayland display handle
		handle: String,
		/// Procol for display handle
		protocol: C::WaylandForeignProtocol,
	},
	/// AppKit window
	///
	/// Note that this should only be used from the main thread
	CapeWindow_AppKit {
		/// AppKit window pointer
		window: *mut std::os::raw::c_void,
	},
}

impl std::default::Default for CapeWindowId {
	fn default() -> Self {
		CapeWindowId::CapeWindow_None
	}
}

unsafe impl Send for CapeWindowId {}
unsafe impl Sync for CapeWindowId {}

impl CapeWindowId {
	/// Get a default CapeWindow with no known or unsupported window type
	///
	/// # Return
	/// Default CapeWindow
	pub fn new() -> CapeWindowId {
		CapeWindowId::CapeWindow_None
	}

	/// Get an X11 CapeWindowId
	///
	/// # Arguments
	/// * `display_string` - Display string
	/// * `xid` - Window ID
	///
	/// # Return
	/// X11 CapeWindow
	pub fn x11(display_string: String, xid: u64) -> CapeWindowId {
		CapeWindowId::CapeWindow_X11 {
			display_string,
			xid,
		}
	}

	/// Get a Wayland CapeWindowId
	///
	/// # Arguments
	/// * `handle` - Wayland display handle
	/// * `protocol` - Protocol for display handle
	///
	/// # Return
	/// Wayland CapeWindowId
	pub fn wayland(handle: String, protocol: C::WaylandForeignProtocol) -> CapeWindowId {
		CapeWindowId::CapeWindow_Wayland { handle, protocol }
	}

	/// Get an AppKit CapeWindowId
	///
	/// Note that this should only be used from the main thread
	///
	/// # Arguments
	/// * `window` - AppKit window pointer
	///
	/// # Return
	/// AppKit CapeWindow
	pub fn appkit(window: *mut std::os::raw::c_void) -> CapeWindowId {
		CapeWindowId::CapeWindow_AppKit { window }
	}

	/// Convert to raw window id
	///
	/// # Return
	/// Raw window id

	pub fn AsRaw(&self) -> CapeWindowIdRaw {
		CapeWindowIdRaw::new(&self)
	}

	/// Convert raw window id to CapeWindowId
	///
	/// # Arguments
	/// * `window_id` - Raw window id
	///
	/// # Return
	/// CapeWindowId

	pub fn FromRaw(window_id: C::CapeWindowId) -> CapeWindowId {
		if window_id.is_null() {
			return CapeWindowId::CapeWindow_None;
		}
		let window = unsafe { *window_id };
		match window.type_ {
			C::eCapeWindowType_CapeWindow_X11 => unsafe {
				if window.data.X11.display_string.is_null() || window.data.X11.xid == 0u64 {
					return CapeWindowId::CapeWindow_None;
				}
				let c_str = std::ffi::CStr::from_ptr(window.data.X11.display_string);
				CapeWindowId::x11(
					c_str.to_str().unwrap_or_default().to_string(),
					window.data.X11.xid as u64,
				)
			},
			C::eCapeWindowType_CapeWindow_Wayland => unsafe {
				if window.data.Wayland.handle.is_null() {
					return CapeWindowId::CapeWindow_None;
				}
				let c_str = std::ffi::CStr::from_ptr(window.data.X11.display_string);
				CapeWindowId::wayland(
					c_str.to_str().unwrap_or_default().to_string(),
					window.data.Wayland.protocol,
				)
			},
			C::eCapeWindowType_CapeWindow_AppKit => unsafe {
				if window.data.AppKit.window.is_null() {
					return CapeWindowId::CapeWindow_None;
				}
				CapeWindowId::appkit(window.data.AppKit.window)
			},
			_ => {
				CapeWindowId::CapeWindow_None //unknown
			}
		}
	}
}

pub struct CapeWindowIdRaw {
	allocated_string: Option<std::ffi::CString>,
	raw: C::CapeWindow,
}

impl CapeWindowIdRaw {
	fn new(w: &CapeWindowId) -> CapeWindowIdRaw {
		let mut allocated_string = None;
		let mut raw = C::CapeWindow {
			type_: C::eCapeWindowType_CapeWindow_None,
			data: C::CapeWindowTypeData {
				X11: C::CapeWindowX11 {
					//dummy
					display_string: std::ptr::null(),
					xid: 0u64,
				},
			},
		};
		match w {
			CapeWindowId::CapeWindow_X11 {
				display_string,
				xid,
			} => {
				if !display_string.is_empty() && *xid != 0u64 {
					allocated_string =
						Some(std::ffi::CString::new(display_string.as_str()).unwrap_or_default());
					raw = C::CapeWindow {
						type_: C::eCapeWindowType_CapeWindow_X11,
						data: C::CapeWindowTypeData {
							X11: C::CapeWindowX11 {
								display_string: allocated_string.as_ref().unwrap().as_ptr(),
								xid: *xid as u64,
							},
						},
					};
				}
			}
			CapeWindowId::CapeWindow_Wayland { handle, protocol } => {
				if !handle.is_empty() {
					allocated_string =
						Some(std::ffi::CString::new(handle.as_str()).unwrap_or_default());
					raw = C::CapeWindow {
						type_: C::eCapeWindowType_CapeWindow_Wayland,
						data: C::CapeWindowTypeData {
							Wayland: C::CapeWindowWayland {
								handle: allocated_string.as_ref().unwrap().as_ptr(),
								protocol: *protocol,
							},
						},
					};
				}
			}
			CapeWindowId::CapeWindow_AppKit { window } => {
				if !window.is_null() {
					raw = C::CapeWindow {
						type_: C::eCapeWindowType_CapeWindow_AppKit,
						data: C::CapeWindowTypeData {
							AppKit: C::CapeWindowAppKit {
								window: window.clone(),
							},
						},
					};
				}
			}
			_ => {}
		};
		CapeWindowIdRaw {
			allocated_string,
			raw,
		}
	}
	pub fn as_native(&self) -> *mut C::CapeWindow {
		&(self.raw) as *const C::CapeWindow as *mut C::CapeWindow
	}
}


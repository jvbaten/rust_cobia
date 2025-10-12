use crate::C;

pub enum CapeWindowIdType {
	CapeWindowId_None,
	CapeWindowId_X11,
	CapeWindowId_Wayland,
	CapeWindowId_COCOA,
}

pub struct CapeWindowId {
	window_id_type: CapeWindowIdType,
	x11_display_ptr: Option<*mut std::os::raw::c_void>, //X11 display pointer
	x11_window_id: Option<u64>, //X11 window id
	wayland_display_ptr: Option<*mut std::os::raw::c_void>, //Wayland display pointer
	wayland_shell_surface_ptr: Option<*mut std::os::raw::c_void>, //Wayland shell surface pointer
	cocoa_window_ptr: Option<*mut std::os::raw::c_void>, //COCOA window pointer
}

impl CapeWindowId {
	pub fn new() -> CapeWindowId {
		CapeWindowId {
			window_id_type: CapeWindowIdType::CapeWindowId_None,
			x11_display_ptr: None,
			x11_window_id: None,
			wayland_display_ptr: None,
			wayland_shell_surface_ptr: None,
			cocoa_window_ptr: None,
		}
	}
	pub fn x11(display_ptr: *mut std::os::raw::c_void, window_id: u64) -> CapeWindowId {
		CapeWindowId {
			window_id_type: CapeWindowIdType::CapeWindowId_X11,
			x11_display_ptr: Some(display_ptr),
			x11_window_id: Some(window_id),
			wayland_display_ptr: None,
			wayland_shell_surface_ptr: None,
			cocoa_window_ptr: None,
		}
	}
	pub fn wayland(display_ptr: *mut std::os::raw::c_void, shell_surface_ptr: *mut std::os::raw::c_void) -> CapeWindowId {
		CapeWindowId {
			window_id_type: CapeWindowIdType::CapeWindowId_Wayland,
			x11_display_ptr: None,
			x11_window_id: None,
			wayland_display_ptr: Some(display_ptr),
			wayland_shell_surface_ptr: Some(shell_surface_ptr),
			cocoa_window_ptr: None,
		}
	}
	pub fn cocoa(window_ptr: *mut std::os::raw::c_void) -> CapeWindowId {
		CapeWindowId {
			window_id_type: CapeWindowIdType::CapeWindowId_COCOA,
			x11_display_ptr: None,
			x11_window_id: None,
			wayland_display_ptr: None,
			wayland_shell_surface_ptr: None,
			cocoa_window_ptr: Some(window_ptr),
		}
	}
}

pub fn CapeWindowIdToRaw(window_id: CapeWindowId) -> C::CapeWindowId {
	match window_id.window_id_type {
		CapeWindowIdType::CapeWindowId_None => {
			C::CapeWindowId {
				type_: C::eCapeWindowIdType_CapeWindowId_None,
				data: C::CapeWindowTypeData {
					X11: C::CapeWindowIdX11 {
						display: std::ptr::null_mut(),
						winID: 0,
					},
				}
			}
		},
		CapeWindowIdType::CapeWindowId_X11 => {
			let x11_display = match window_id.x11_display_ptr {
				Some(ptr) => ptr,
				None => std::ptr::null_mut(),
			};
			let x11_window_id = match window_id.x11_window_id {
				Some(id) => id,
				None => 0,
			};
			C::CapeWindowId {
				type_: C::eCapeWindowIdType_CapeWindowId_X11,
				data: C::CapeWindowTypeData {
					X11: C::CapeWindowIdX11 {
						display: x11_display,
						winID: x11_window_id,
					},
				},
			}
		},
		CapeWindowIdType::CapeWindowId_Wayland => {
			let wayland_display = match window_id.wayland_display_ptr {
				Some(ptr) => ptr,
				None => std::ptr::null_mut(),
			};
			let wayland_shell_surface = match window_id.wayland_shell_surface_ptr {
				Some(ptr) => ptr,
				None => std::ptr::null_mut(),
			};
			C::CapeWindowId {
				type_: C::eCapeWindowIdType_CapeWindowId_Wayland,
				data: C::CapeWindowTypeData {
					Wayland: C::CapeWindowIdWayland {
						display: wayland_display,
						shellsurface: wayland_shell_surface,
					},
				},
			}
		},
		CapeWindowIdType::CapeWindowId_COCOA => {
			let cocoa_window = match window_id.cocoa_window_ptr {
				Some(ptr) => ptr,
				None => std::ptr::null_mut(),
			};
			C::CapeWindowId {
				type_: C::eCapeWindowIdType_CapeWindowId_COCOA,
				data: C::CapeWindowTypeData {
					COCOA: C::CapeWindowIdCOCOA {
						window: cocoa_window,
					},
				},
			}
		},
	}
}

pub fn CapeWindowIdFromRaw(window_id: C::CapeWindowId) -> CapeWindowId {
	match window_id.type_ {
		C::eCapeWindowIdType_CapeWindowId_X11 => {
			unsafe {CapeWindowId::x11(window_id.data.X11.display, window_id.data.X11.winID)}
		},
		C::eCapeWindowIdType_CapeWindowId_Wayland => {
			unsafe {CapeWindowId::wayland(window_id.data.Wayland.display, window_id.data.Wayland.shellsurface)}
		},
		C::eCapeWindowIdType_CapeWindowId_COCOA => {
			unsafe {CapeWindowId::cocoa(window_id.data.COCOA.window)}
		},
		_  => {
			CapeWindowId::new() //unknown
		},
	}


}

//! # Parenting the editor window to the host window on X11 and Wayland
//!
//! The flowsheet host passes its window to the unit as a [`cobia::CapeWindowId`], which the
//! unit forwards to this editor process as a [`ParentWindow`]. Here the editor's main window is
//! made a transient child of that host window, so that the window manager / compositor keeps
//! the editor on top of its parent.
//!
//! * On X11, the `WM_TRANSIENT_FOR` hint is set on the editor window.
//! * On Wayland, the host has exported its toplevel surface using the XDG foreign protocol
//!   (see `unit_operation_runner/src-tauri/src/window.rs` for the exporting side). Here the
//!   exported handle is imported, and the editor's surface is set as its child.

use crate::edit_dialog_messages::ParentWindow;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use wayland_client::Proxy;
use wayland_client::protocol::{wl_registry::WlRegistry, wl_surface::WlSurface};
use wayland_protocols::xdg::foreign::zv1::client::{
	zxdg_imported_v1::ZxdgImportedV1, zxdg_importer_v1::ZxdgImporterV1,
};
use wayland_protocols::xdg::foreign::zv2::client::{
	zxdg_imported_v2::ZxdgImportedV2, zxdg_importer_v2::ZxdgImporterV2,
};

/// Makes `window` a child of the host window described by `parent`; part to be
/// called before the window is shown.
///
/// On X11 the `WM_TRANSIENT_FOR` hint must be set before the window is mapped, as
/// window managers may only read it at map time. GTK only creates the native window
/// once it is shown, so the window is realized (created, but not mapped) here first.
/// The window is also typed as a dialog, as GTK otherwise explicitly marks it as a
/// normal window, which makes some window managers ignore the transient relation.
///
/// Must be called on the main thread.
///
/// # Arguments
/// * `window` - The editor's main window, not yet shown.
/// * `parent` - The host window, as received from the unit.
pub(crate) fn set_parent_window_before_show(window: &tauri::WebviewWindow, parent: &ParentWindow) {
	let (Some(display_name), Some(parent_xid)) = (&parent.x11_display, parent.x11_window_id)
	else {
		return;
	};
	if let Ok(gtk_window) = window.gtk_window() {
		use gtk::prelude::{GtkWindowExt, WidgetExt};
		//mark the window as a dialog; window managers (e.g. X410) may only keep a
		//transient window on top of its parent if it is not a normal window
		gtk_window.set_type_hint(gtk::gdk::WindowTypeHint::Dialog);
		gtk_window.realize();
	}
	if let Ok(wnd) = window.window_handle() {
		if let RawWindowHandle::Xlib(wnd) = wnd.as_raw() {
			set_x11_parent(display_name, wnd.window, parent_xid);
		}
	}
}

/// Makes `window` a child of the host window described by `parent`; part to be
/// called after the window is shown.
///
/// On Wayland the editor's surface must be an `xdg_toplevel` (i.e. the window must be
/// mapped) before it can be given a parent; using it any earlier is a protocol error
/// that would break GTK's connection. Tauri shows windows asynchronously on its event
/// loop, so this function polls (in a separate thread) until the window is visible.
/// Window state is queried via Tauri, which takes care of doing so on the main thread.
///
/// # Arguments
/// * `window` - The editor's main window; shown, or about to be shown.
/// * `parent` - The host window, as received from the unit.
pub(crate) fn set_parent_window_after_show(window: tauri::WebviewWindow, parent: ParentWindow) {
	let Some(handle) = parent.wayland_external_id else {
		return;
	};
	std::thread::spawn(move || {
		//wait for at most 10 seconds for the window to be shown
		for _ in 0..200 {
			if window.is_visible().unwrap_or(false) {
				if let (Ok(wnd), Ok(dsp)) = (window.window_handle(), window.display_handle()) {
					if let (RawWindowHandle::Wayland(wnd), RawDisplayHandle::Wayland(dsp)) =
						(wnd.as_raw(), dsp.as_raw())
					{
						set_wayland_parent(
							dsp.display.as_ptr(),
							wnd.surface.as_ptr(),
							&handle,
							parent.wayland_foreign_protocol_version,
						);
					}
				}
				return;
			}
			std::thread::sleep(std::time::Duration::from_millis(50));
		}
	});
}

/// Sets the `WM_TRANSIENT_FOR` hint of the editor window to the host window.
///
/// In addition, the `WM_CLIENT_LEADER` of the editor window is set to that of the host
/// window (or removed if the host window has none), so that the editor window is seen as
/// part of the host application; some window managers (e.g. X410) only keep a transient
/// window on top of its parent if both have the same client leader, which is naturally
/// the case for in-process dialogs, but not for this out-of-process editor.
///
/// A private connection to the host's X display is used, as window IDs are global to the
/// X server. Closing the connection flushes the requests.
///
/// # Arguments
/// * `display_name` - The host's X11 display name (e.g. `:0`).
/// * `child_xid` - The X11 window ID of the editor window.
/// * `parent_xid` - The X11 window ID of the host window.
fn set_x11_parent(display_name: &str, child_xid: std::os::raw::c_ulong, parent_xid: u64) {
	use x11::xlib;
	let Ok(display_name) = std::ffi::CString::new(display_name) else {
		return;
	};
	unsafe {
		let display = xlib::XOpenDisplay(display_name.as_ptr());
		if display.is_null() {
			return;
		}
		let parent_xid = parent_xid as xlib::Window;
		xlib::XSetTransientForHint(display, child_xid, parent_xid);
		//share the client leader of the parent
		let client_leader = xlib::XInternAtom(display, c"WM_CLIENT_LEADER".as_ptr(), xlib::False);
		let mut actual_type: xlib::Atom = 0;
		let mut actual_format: std::os::raw::c_int = 0;
		let mut item_count: std::os::raw::c_ulong = 0;
		let mut bytes_after: std::os::raw::c_ulong = 0;
		let mut data: *mut std::os::raw::c_uchar = std::ptr::null_mut();
		let status = xlib::XGetWindowProperty(
			display,
			parent_xid,
			client_leader,
			0,
			1,
			xlib::False,
			xlib::XA_WINDOW,
			&mut actual_type,
			&mut actual_format,
			&mut item_count,
			&mut bytes_after,
			&mut data,
		);
		if status == xlib::Success as std::os::raw::c_int
			&& actual_type == xlib::XA_WINDOW
			&& actual_format == 32
			&& item_count == 1
		{
			//format 32 data is returned as an array of longs
			xlib::XChangeProperty(
				display,
				child_xid,
				client_leader,
				xlib::XA_WINDOW,
				32,
				xlib::PropModeReplace,
				data,
				1,
			);
		} else {
			xlib::XDeleteProperty(display, child_xid, client_leader);
		}
		if !data.is_null() {
			xlib::XFree(data as *mut _);
		}
		xlib::XCloseDisplay(display);
	}
}

/// Wayland globals of interest, collected from the registry; also serves as the
/// dispatch state for the event queue on which the importer objects live.
#[derive(Default)]
struct WaylandImporterGlobals {
	/// Name and version of the `zxdg_importer_v2` global, if announced.
	importer_v2: Option<(u32, u32)>,
	/// Name and version of the `zxdg_importer_v1` global, if announced.
	importer_v1: Option<(u32, u32)>,
}

/// Wayland objects that must outlive the parent/child relationship.
struct WaylandParentObjects {
	_connection: wayland_client::Connection,
	_event_queue: wayland_client::EventQueue<WaylandImporterGlobals>,
	_registry: WlRegistry,
	_imported_v2: Option<ZxdgImportedV2>,
	_imported_v1: Option<ZxdgImportedV1>,
}

/// Imports the host's exported toplevel and sets it as the parent of the editor's surface.
///
/// The editor's Wayland connection (owned by GTK) is shared, using a private event queue.
/// The imported object must stay alive for as long as the relationship should hold, and
/// the event queue must not be destroyed while objects are still attached to it; as the
/// editor process terminates once the dialog closes, these objects are simply leaked.
///
/// # Arguments
/// * `display` - The `wl_display` of the editor's connection.
/// * `surface` - The `wl_surface` of the editor window; must be an `xdg_toplevel`.
/// * `handle` - The handle under which the host exported its toplevel.
/// * `protocol_version` - The XDG foreign protocol version used by the host to export
///   the handle (1 or 2); if not known, v2 is preferred.
fn set_wayland_parent(
	display: *mut std::ffi::c_void,
	surface: *mut std::ffi::c_void,
	handle: &str,
	protocol_version: Option<u32>,
) {
	let backend = unsafe {
		wayland_backend::sys::client::Backend::from_foreign_display(
			display as *mut wayland_sys::client::wl_display,
		)
	};
	let connection = wayland_client::Connection::from_backend(backend);
	let Ok(surface_id) = (unsafe {
		wayland_backend::sys::client::ObjectId::from_ptr(
			WlSurface::interface(),
			surface as *mut wayland_sys::client::wl_proxy,
		)
	}) else {
		return;
	};
	let Ok(surface) = WlSurface::from_id(&connection, surface_id) else {
		return;
	};
	let mut globals = WaylandImporterGlobals::default();
	let mut event_queue = connection.new_event_queue();
	let qhandle = event_queue.handle();
	let registry = connection.display().get_registry(&qhandle, ());
	if event_queue.roundtrip(&mut globals).is_err() {
		return;
	}
	let use_v2 = protocol_version != Some(1);
	let use_v1 = protocol_version != Some(2);
	let mut imported_v2 = None;
	let mut imported_v1 = None;
	//if the compositor does not support the protocol, the window simply remains unparented
	if let (true, Some((name, version))) = (use_v2, globals.importer_v2) {
		let importer = registry.bind::<ZxdgImporterV2, _, _>(
			name,
			version.min(ZxdgImporterV2::interface().version),
			&qhandle,
			(),
		);
		let imported = importer.import_toplevel(handle.to_string(), &qhandle, ());
		imported.set_parent_of(&surface);
		importer.destroy(); //does not affect the imported object
		imported_v2 = Some(imported);
	} else if let (true, Some((name, version))) = (use_v1, globals.importer_v1) {
		let importer = registry.bind::<ZxdgImporterV1, _, _>(
			name,
			version.min(ZxdgImporterV1::interface().version),
			&qhandle,
			(),
		);
		let imported = importer.import(handle.to_string(), &qhandle, ());
		imported.set_parent_of(&surface);
		importer.destroy(); //does not affect the imported object
		imported_v1 = Some(imported);
	}
	//make sure the requests are processed
	let _ = event_queue.roundtrip(&mut globals);
	std::mem::forget(WaylandParentObjects {
		_connection: connection,
		_event_queue: event_queue,
		_registry: registry,
		_imported_v2: imported_v2,
		_imported_v1: imported_v1,
	});
}

impl wayland_client::Dispatch<WlRegistry, ()> for WaylandImporterGlobals {
	fn event(
		globals: &mut Self,
		_: &WlRegistry,
		event: wayland_client::protocol::wl_registry::Event,
		_: &(),
		_: &wayland_client::Connection,
		_: &wayland_client::QueueHandle<Self>,
	) {
		if let wayland_client::protocol::wl_registry::Event::Global {
			name,
			interface,
			version,
		} = event
		{
			if interface == "zxdg_importer_v2" {
				globals.importer_v2 = Some((name, version));
			} else if interface == "zxdg_importer_v1" {
				globals.importer_v1 = Some((name, version));
			}
		}
	}
}

impl wayland_client::Dispatch<ZxdgImporterV2, ()> for WaylandImporterGlobals {
	fn event(
		_: &mut Self,
		_: &ZxdgImporterV2,
		_: wayland_protocols::xdg::foreign::zv2::client::zxdg_importer_v2::Event,
		_: &(),
		_: &wayland_client::Connection,
		_: &wayland_client::QueueHandle<Self>,
	) {
		//no events
	}
}

impl wayland_client::Dispatch<ZxdgImporterV1, ()> for WaylandImporterGlobals {
	fn event(
		_: &mut Self,
		_: &ZxdgImporterV1,
		_: wayland_protocols::xdg::foreign::zv1::client::zxdg_importer_v1::Event,
		_: &(),
		_: &wayland_client::Connection,
		_: &wayland_client::QueueHandle<Self>,
	) {
		//no events
	}
}

impl wayland_client::Dispatch<ZxdgImportedV2, ()> for WaylandImporterGlobals {
	fn event(
		_: &mut Self,
		_: &ZxdgImportedV2,
		_: wayland_protocols::xdg::foreign::zv2::client::zxdg_imported_v2::Event,
		_: &(),
		_: &wayland_client::Connection,
		_: &wayland_client::QueueHandle<Self>,
	) {
		//destroyed: the handle was invalid or the host window went away; nothing to do
	}
}

impl wayland_client::Dispatch<ZxdgImportedV1, ()> for WaylandImporterGlobals {
	fn event(
		_: &mut Self,
		_: &ZxdgImportedV1,
		_: wayland_protocols::xdg::foreign::zv1::client::zxdg_imported_v1::Event,
		_: &(),
		_: &wayland_client::Connection,
		_: &wayland_client::QueueHandle<Self>,
	) {
		//destroyed: the handle was invalid or the host window went away; nothing to do
	}
}

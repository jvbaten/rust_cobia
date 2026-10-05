//! # Distillation shortcut unit operation editor
//!
//! The editor is implemented using Tauri v2, and runs
//! out-of-process from the CAPE-OPEN compliant flowsheeting environment,
//! as Tauri (or rather tao) does not like running in a foreign thread,
//! which is not something the CAPE-OPEN unit operation has control over.
//!
//! In addition when running in-process, tao tends to leave input handling
//! hooks after the main dialog terminates, and the easiest way to get rid of those is
//! to terminate the process after editing is done.
//!
//! This crate is the client side of the IPC protocol defined in
//! [`edit_dialog_messages`]: it connects back to the unit ("server") process that
//! spawned it, forwards [`UpdateMessage`]s from the server to the Tauri frontend as
//! window events (`update-status`, `update-content`, `update-streams`), and forwards
//! frontend actions to the server as [`InputMessage`]s via the `request_update` and
//! `data_entry` Tauri commands.

mod edit_dialog_messages;
#[cfg(not(target_os = "windows"))]
mod parent_window_posix;
use edit_dialog_messages::*;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
#[cfg(target_os = "windows")]
use raw_window_handle::HasWindowHandle;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

/// Tauri-managed state holding the channel back to the server process. Shared with
/// the frontend's command handlers via [`tauri::State`].
struct EditData {
	/// Sender used to forward [`InputMessage`]s to the server process.
	tx: IpcSender<InputMessage>,
}

impl EditData {
	/// # Arguments
	/// * `tx` - The IPC sender connected to the server process.
	///
	/// # Returns
	/// A new [`EditData`] wrapping `tx`.
	fn new(tx: IpcSender<InputMessage>) -> Self {
		Self { tx }
	}
}

/// Tauri command invoked by the frontend once its UI has loaded, requesting the
/// server send the dialog's initial state.
///
/// # Arguments
/// * `state` - The managed [`EditData`], used to reach the server's IPC sender.
///
/// # Returns
/// Nothing; sends [`InputMessage::InitialUpdate`] to the server. Send failures
/// (e.g. the server process having exited) are silently ignored.
#[tauri::command]
fn request_update(state: State<'_, Mutex<EditData>>) {
	let _ = state.lock().unwrap().tx.send(InputMessage::InitialUpdate);
}

/// Tauri command invoked by the frontend whenever the user changes a control's
/// value in the dialog.
///
/// # Arguments
/// * `state` - The managed [`EditData`], used to reach the server's IPC sender.
/// * `control_id` - Identifier of the edited control (matches an ID in the dialog's
///   HTML).
/// * `value` - The control's new value, as text.
///
/// # Returns
/// Nothing; sends an [`InputMessage::DataEntry`] to the server. Send failures
/// (e.g. the server process having exited) are silently ignored.
#[tauri::command]
fn data_entry(state: State<'_, Mutex<EditData>>, control_id: String, value: String) {
	let _ = state
		.lock()
		.unwrap()
		.tx
		.send(InputMessage::DataEntry(DataEntry { control_id, value }));
}

/// Entry point for the editor's Tauri application, invoked by `main`.
///
/// Parses the server's IPC one-shot server name from the command line, connects
/// to it, and completes the IPC handshake by
/// creating a fresh channel and sending it to the server via
/// [`InputMessage::SetChannel`].
///
/// Once connected, builds the Tauri application, registers commands, and spawns
/// a background thread that receives [`UpdateMessage`]s from the server.
///
/// # Arguments
/// None; the server's connection name is read from `std::env::args()`.
///
/// # Returns
/// Never returns normally: `app.run` blocks for the lifetime of the Tauri event
/// loop, and this process is expected to be terminated (see the module-level docs)
/// once the dialog closes. Setup failures instead terminate the process via
/// `std::process::exit(-1)`.
pub fn show_dialog() {
	//parse command line
	let mut server_name = None;
	let mut msg = None;
	let mut first = true;
	for arg in std::env::args() {
		if first {
			first = false;
			#[cfg(target_os = "windows")]
			continue;
			#[cfg(not(target_os = "windows"))]
			{
				//check if this is indeed the executable name
				if arg.ends_with("distillation_shortcut_unit_editor")
					|| arg.ends_with("distillation_shortcut_unit_editor.exe")
				{
					continue;
				}
			}
		}
		if server_name.is_some() {
			msg = Some("Multiple server names specified".to_string());
		}
		server_name = Some(arg);
	}
	let tx: Option<IpcSender<InputMessage>> = match server_name {
		Some(ref name) => match IpcSender::connect(name.clone()) {
			Ok(sender) => Some(sender),
			Err(ee) => {
				msg = Some(format!("Failed to connect to server: {}", ee));
				None
			}
		},
		None => {
			msg = Some("No server name specified".to_string());
			None
		}
	};
	let mut rx: Option<IpcReceiver<UpdateMessage>> = None;
	if msg.is_none() {
		//create receiver
		match ipc::channel() {
			Ok((tx1, rx1)) => {
				rx = Some(rx1);
				match tx
					.as_ref()
					.unwrap()
					.send(InputMessage::SetChannel { channel: tx1 })
				{
					Ok(_) => {}
					Err(e) => {
						msg = Some(format!("Failed to send message to server: {}", e));
					}
				}
			}
			Err(e) => {
				msg = Some(format!("Failed to create IPC channel: {}", e));
			}
		};
	}
	let window_msg = if msg.is_none() {
		//the first message that comes in is the parent window handle, read it
		match rx.as_ref().unwrap().recv() {
			Ok(UpdateMessage::ParentWindow { parent_window }) => Some(parent_window),
			Ok(_) => {
				msg = Some("Error: Expected ParentWindow message from server".to_string());
				None
			}
			Err(e) => {
				msg = Some(format!(
					"Error: Failed to receive ParentWindow message from server: {}",
					e
				));
				None
			}
		}
	} else {
		None
	};
	if let Some(msg) = msg {
		#[cfg(target_os = "windows")]
		{
			use windows::{
				Win32::UI::WindowsAndMessaging::{MB_ICONHAND, MB_OK, MessageBoxW},
				core::HSTRING,
			};
			unsafe {
				MessageBoxW(
					None,
					&HSTRING::from(msg),
					&HSTRING::from("Distillation Shortcut Unit Editor"),
					MB_OK | MB_ICONHAND,
				);
			}
		}
		#[cfg(not(target_os = "windows"))]
		{
			eprintln!("Error: {}", msg);
		}
		std::process::exit(-1);
	}
	//ok to proceed
	let window_msg = window_msg.unwrap();
	let tx = tx.unwrap();
	let rx = rx.unwrap();
	//set up app
	let app = tauri::Builder::default()
		.setup(move |app| {
			app.manage(Mutex::new(EditData::new(tx)));
			//set up the main window
			let window = tauri::webview::WebviewWindowBuilder::new(
				app,
				"main",
				tauri::WebviewUrl::App("index.html".into()),
			)
			.title("Distillation Shortcut Unit")
			.inner_size(600.0, 600.0)
			.minimizable(false)
			.visible(false)
			.build()?;

			#[cfg(target_os = "windows")]
			if let Some(hwnd) = window_msg.windows_hwnd {
				if let Ok(rawwnd) = window.window_handle() {
					if let raw_window_handle::RawWindowHandle::Win32(raw) = rawwnd.as_raw() {
                        //we would like to make the window be a owned by the parent window, using
                        //SetWindowLongPtrW with GWLP_HWNDPARENT, so that the window always stays on top of
                        //its owner. This seems to cause dead locks in Tauri, so we stay away from it. Instead we 
                        //bring the window to the foreground each time we find the parent or one of its
                        //ancestors on top of the z-order. This check is done periodically in a separate thread.
                        //
                        //note that WebviewWindowBuilder::owner_raw() also causes the edit window to hang.
						let _thread_handle = std::thread::spawn(move || {
							let tauri_wnd = windows::Win32::Foundation::HWND {
								0: raw.hwnd.get() as *mut _,
							};
							struct HashWindow(windows::Win32::Foundation::HWND);
							impl HashWindow {
								fn new(hwnd: windows::Win32::Foundation::HWND) -> Self {
									Self(hwnd)
								}
							}
							impl std::hash::Hash for HashWindow {
								fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
									state.write_usize(self.0.0 as usize);
								}
							}
							impl PartialEq for HashWindow {
								fn eq(&self, other: &Self) -> bool {
									self.0.0 == other.0.0
								}
							}
							impl Eq for HashWindow {}
							let mut ancestors = std::collections::HashSet::new();
							let mut current =
								windows::Win32::Foundation::HWND { 0: hwnd as *mut _ };
							ancestors.insert(HashWindow::new(current));
							loop {
								if let Ok(parent) = unsafe {
									windows::Win32::UI::WindowsAndMessaging::GetParent(current)
								} {
									current = parent;
									ancestors.insert(HashWindow::new(current));
									continue;
								}
								if let Ok(owner) = unsafe {
									windows::Win32::UI::WindowsAndMessaging::GetWindow(
										current,
										windows::Win32::UI::WindowsAndMessaging::GW_OWNER,
									)
								} {
									current = owner;
									ancestors.insert(HashWindow::new(current));
									continue;
								}
								break;
							}
							//now periodically check if the foreground window is one of the ancestors, and if so, bring the Tauri window to the foreground
							loop {
								std::thread::sleep(std::time::Duration::from_millis(300));
								let foreground = unsafe {
									windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow()
								};
								if ancestors.contains(&HashWindow::new(foreground)) {
									unsafe {
										let _=windows::Win32::UI::WindowsAndMessaging::SetForegroundWindow(tauri_wnd);
									}
								}
							}
						});
					}
				}
			}

			//make the window a child of the host window
			#[cfg(not(target_os = "windows"))]
			parent_window_posix::set_parent_window_before_show(&window, &window_msg);

			window.show()?;

			#[cfg(not(target_os = "windows"))]
			parent_window_posix::set_parent_window_after_show(window, window_msg);

			Ok(())
		})
		.invoke_handler(tauri::generate_handler![request_update, data_entry])
		.build(tauri::generate_context!())
		.expect("error while building tauri application");
	//a thread to receive messages from the server and send them to the Tauri app
	let apph = app.handle().clone();
	let _thread_handle = std::thread::spawn(move || {
		//process incoming messages
		while let Ok(msg) = rx.recv() {
			match msg {
				UpdateMessage::UpdateStatus { status } => {
					let _ = apph.emit("update-status", status);
				}
				UpdateMessage::UpdateContent { content } => {
					let _ = apph.emit("update-content", content);
				}
				UpdateMessage::UpdateStreams { streams } => {
					let _ = apph.emit("update-streams", streams);
				}
				_ => {
					//unexpected, ignore
				}
			}
		}
	});
	app.run(|_app_handle, _event| {
		//no event processing needed
	});
}

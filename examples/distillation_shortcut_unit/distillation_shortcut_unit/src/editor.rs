//! # CAPE-OPEN editor entry point for the distillation shortcut unit
//!
//! Implements the unit's `Edit()` behavior by launching the out-of-process Tauri
//! editor (`distillation_shortcut_unit_editor.exe`) and driving it over an
//! `ipc_channel` connection, using the message protocol defined in
//! [`crate::edit_dialog_messages`].
//!
//! The unit is the IPC server: it starts an [`IpcOneShotServer`] and passes its name
//! to the spawned editor process on the command line, then waits for the editor to
//! connect and send [`InputMessage::SetChannel`], which hands back a sender the unit
//! uses to push [`UpdateMessage`]s. From then on the unit answers editor-initiated
//! messages ([`InputMessage::InitialUpdate`], [`InputMessage::DataEntry`]) until the
//! editor process disconnects (closing its dialog), at which point [`edit`] returns
//! whether any parameter was modified.
use cobia::{*,prelude::*};
use crate::{distillation_shortcut_unit::DistillationShortcutUnit,edit_dialog_messages::*};
use ipc_channel::ipc::{IpcOneShotServer,IpcSender};

#[cfg(target_os = "windows")]
use ipc_channel::TryRecvError;

/// Reduces a [`COBIAError`] to a short, user-presentable message, discarding any
/// error cause chain so it fits on the editor's single-line status display.
///
/// # Arguments
/// * `e` - The error to format.
///
/// # Returns
/// The error's display message, with any cause chain or interface-specific detail
/// stripped off.
fn short_error(e: COBIAError) -> String {
    match e {
        COBIAError::Message(msg) => msg,
        COBIAError::MessageWithCause(msg, _cause) => msg,
        COBIAError::Code(_) => e.into(),
        COBIAError::CAPEOPEN(ref err) => {
            if let Ok(msg) = err.get_error_text() {
                msg
            } else {
                e.into()
            }
        }
    }
}

/// Builds a snapshot of the unit's current parameter values for the editor dialog.
///
/// # Arguments
/// * `unit` - The unit to read parameter values from.
///
/// # Returns
/// A [`Content`] populated from `unit`'s current parameter and result values.
fn get_content(unit: &DistillationShortcutUnit) -> Content{
    Content {
        unit_name: unit.get_name(),
        compound_list: unit.get_compound_list(),
        unit_description: unit.get_description(),
        light_key_compound: unit.get_light_key_compound(),
        heavy_key_compound: unit.get_heavy_key_compound(),
        light_key_compound_recovery: unit.get_light_key_compound_recovery(),
        heavy_key_compound_recovery: unit.get_heavy_key_compound_recovery(),
        reflux_ratio_factor: unit.get_reflux_ratio_factor(),
        maximum_iterations: unit.get_maximum_iterations(),
        convergence_tolerance: unit.get_convergence_tolerance(),
        number_of_stages: unit.get_number_of_stages(),
        reflux_ratio: unit.get_reflux_ratio(),
        feed_stage_location: unit.get_feed_stage_location(),
    }
}

/// Builds the stream info table shown in the editor dialog: one column per connected
/// port (feed, distillate, bottoms) and one row per property. The first row holds the
/// connected stream names; if at least one port is connected, further rows are added
/// for temperature, pressure, and per-compound feed flow with the corresponding
/// distillate/bottoms flow and recovery percentage.
///
/// Missing values (unconnected ports, properties that fail to evaluate) are rendered
/// as `"N/A"` or an empty string rather than causing an error, since this table is
/// informational only.
///
/// # Arguments
/// * `unit` - The unit whose connected ports/streams are inspected.
///
/// # Returns
/// A list of table rows, each row being a list of display strings: a label cell
/// followed by one cell per port (feed, distillate, bottoms).
fn get_streams(unit: &DistillationShortcutUnit) -> Vec::<Vec<String>> {
    //fill the streams table
    let ports = unit.get_ports();
    //port name and connected objects
    let mut streams =Vec::<Option<cape_open_1_2::CapeThermoMaterial>>::with_capacity(3);
    let mut port_info = Vec::<Vec<String>>::new();
    for port in ports {
        //get stream
        if let Ok(stream) = port.get_connected_object() {
            match cape_open_1_2::CapeThermoMaterial::from_object(&stream) {
                Ok(s) => {
                    streams.push(Some(s));
                }
                Err(_) => {
                    streams.push(None);
                }
            }
        } else {
            streams.push(None);
        }
    }
    //stream connections
    let mut stream_names = Vec::<String>::with_capacity(4);
    let mut any = false;
    stream_names.push("Stream".into());
    for stream in &streams {
        if let Some(stream) = stream {
            //get stream name
            any = true;
            let mut name = "<Unnamed>".to_string();
            if let Ok(iden) = cape_open_1_2::CapeIdentification::from_object(stream) {
                let mut stream_name = cobia::CapeStringImpl::new();
                if iden.get_component_name(&mut stream_name).is_ok() {
                    name = stream_name.to_string();
                }
            }
            stream_names.push(name);
        } else {
            stream_names.push("<Not connected>".into());
        }
    }
    port_info.push(stream_names);
    if any {
        let mut values = cobia::CapeArrayRealVec::new();
        let str_no_basis = cobia::CapeStringImpl::new();
        let str_mass_basis = cobia::CapeStringImpl::from("mass");
        //stream temperature
        let mut stream_temperature = Vec::<String>::with_capacity(4);
        stream_temperature.push("Temperature / [°C]".into());
        let str_temperature = cobia::CapeStringImpl::from("temperature");
        for stream in &streams {
            if let Some(stream) = stream {
                match stream.get_overall_prop(
                    &str_temperature,
                    &str_no_basis,
                    &mut values,
                ) {
                    Ok(()) => {
                        if values.size() == 1 {
                            stream_temperature
                                .push(format!("{:.2}", values[0] - 273.15));
                        } else {
                            stream_temperature.push("N/A".into());
                        }
                    }
                    Err(_) => {
                        stream_temperature.push("N/A".into());
                    }
                }
            } else {
                stream_temperature.push(String::new());
            }
        }
        port_info.push(stream_temperature);
        //stream pressure
        let mut stream_pressure = Vec::<String>::with_capacity(4);
        stream_pressure.push("Pressure / [bar]".into());
        let str_pressure = cobia::CapeStringImpl::from("pressure");
        for stream in &streams {
            if let Some(stream) = stream {
                match stream.get_overall_prop(&str_pressure, &str_no_basis, &mut values)
                {
                    Ok(()) => {
                        if values.size() == 1 {
                            stream_pressure.push(format!("{:.3}", values[0] * 1e-5));
                        } else {
                            stream_pressure.push("N/A".into());
                        }
                    }
                    Err(_) => {
                        stream_pressure.push("N/A".into());
                    }
                }
            } else {
                stream_pressure.push(String::new());
            }
        }
        port_info.push(stream_pressure);
        //component flows and recoveries
        let comp_list = unit.get_compound_list();
        let str_flow = cobia::CapeStringImpl::from("flow");
        let mut feed_rates = cobia::CapeArrayRealVec::new();
        if let Some(ref feed) = streams[0] {
            let _ = feed.get_overall_prop(&str_flow, &str_mass_basis, &mut feed_rates);
        }
        if feed_rates.size() == comp_list.len() {
            let mut distillate_rates = cobia::CapeArrayRealVec::new();
            let mut bottoms_rates = cobia::CapeArrayRealVec::new();
            if let Some(ref distillate) = streams[1] {
                let _ = distillate.get_overall_prop(
                    &str_flow,
                    &str_mass_basis,
                    &mut distillate_rates,
                );
            }
            if let Some(ref bottoms) = streams[2] {
                let _ = bottoms.get_overall_prop(
                    &str_flow,
                    &str_mass_basis,
                    &mut bottoms_rates,
                );
            }
            //iterate over compounds by index
            comp_list.iter().enumerate().for_each(|(i, comp)| {
                let mut stream_comp = Vec::<String>::with_capacity(4);
                stream_comp.push("F[".to_string() + comp + "] / [kg/s]");
                stream_comp.push(format!("{:.3}", feed_rates[i]));
                if distillate_rates.size() == comp_list.len() {
                    if distillate_rates[i] <= 0.0 {
                        stream_comp.push("0".into());
                    } else {
                        stream_comp.push(format!(
                            "{:.3} ({:.2}%)",
                            distillate_rates[i],
                            100.0 * distillate_rates[i] / feed_rates[i]
                        ));
                    }
                } else {
                    stream_comp.push(String::new());
                }
                if bottoms_rates.size() == comp_list.len() {
                    if bottoms_rates[i] <= 0.0 {
                        stream_comp.push("0".into());
                    } else {
                        stream_comp.push(format!(
                            "{:.3} ({:.2}%)",
                            bottoms_rates[i],
                            100.0 * bottoms_rates[i] / feed_rates[i]
                        ));
                    }
                } else {
                    stream_comp.push(String::new());
                }
                port_info.push(stream_comp);
            });
        }
    }
    port_info
}

/// Runs the unit's internal validation and turns the result into a [`Status`] message
/// for display in the editor dialog (a success message on the happy path, or the
/// [`short_error`]-formatted validation failure).
///
/// # Arguments
/// * `unit` - The unit to validate. Taken mutably because `validate_internal` is a
///   method on the mutable unit interface, though this call does not otherwise
///   change unit state.
///
/// # Returns
/// A [`Status`] with `error: false` and a success message if validation passed, or
/// `error: true` and the validation failure message otherwise.
 fn get_status(unit: &mut DistillationShortcutUnit) -> Status {
    match unit.validate_internal() {
        Ok(()) => {
            Status {
                text: "Specification is complete".into(),
                error: false,
            }
        },
        Err(e) => {
            Status {
                text: short_error(e),
                error: true,
            }
        }
    }
}

/// Applies a single edited control's value to the unit.
///
/// Dispatches on `entry.control_id` to find the matching setter, parsing and
/// validating `entry.value` first (integers for `maximum_iterations`, floats for the
/// other numeric parameters, and membership in the unit's compound list for the key
/// compound selectors). Values that are unchanged from the current setting are
/// skipped without calling the setter.
///
/// # Arguments
/// * `unit` - The unit to update.
/// * `entry` - The control ID and new value reported by the editor dialog.
///
/// # Returns
/// * `Ok(true)` - The value was valid and different from the current setting, and
///   was applied to `unit`.
/// * `Ok(false)` - The value was valid but matched the current setting, so no
///   change was made.
/// * `Err(String)` - A user-presentable message explaining why `entry.value` was
///   rejected (parse failure, unknown compound, unrecognized control ID, or a
///   validation error from the setter itself).
fn data_entry(unit: &mut DistillationShortcutUnit, entry: &DataEntry) -> Result<bool, String> {
    let mut modified=false;
    let (control_id, value) = (&entry.control_id, &entry.value);
    match control_id.as_str() {
        "unit_name" => {
            if value.is_empty() {
                return Err("Unit name cannot be empty".into());
            }
            if value != unit.get_name().as_str() {
                unit.set_name(value);
                modified = true;
            }
        }
        "unit_description" => {
            if value != unit.get_description().as_str() {
                unit.set_description(&value);
                modified = true;
            }
        }
        "light_key_compound" => {
            if value != unit.get_light_key_compound().as_str() {
                if !unit.get_compound_list().iter().any(|comp| comp.as_str() == value) {
                    return Err(format!("Compound {} is not defined", value));
                } 
                match unit.set_light_key_compound(value) {
                    Ok(()) => {
                        modified = true;
                    }
                    Err(e) => {
                        return Err(short_error(e));
                    }
                }
            }
        }
        "heavy_key_compound" => {
            let new_comp: String = value.into();
            if new_comp != unit.get_heavy_key_compound().as_str() {
                if !unit.get_compound_list().iter().any(|comp| comp.as_str() == new_comp) {
                    return Err(format!("Compound {} is not defined", new_comp));
                } else {
                    match unit.set_heavy_key_compound(&new_comp) {
                        Ok(()) => {
                            modified = true;
                        }
                        Err(e) => {
                            return Err(short_error(e));
                        }
                    }
                }
            }
        }
        "maximum_iterations" => {
            match value.parse::<i32>() {
                Ok(value) => {
                    if value != unit.get_maximum_iterations() {
                        match unit.set_maximum_iterations(value) {
                            Ok(()) => {
                                modified = true;
                            }
                            Err(e) => {
                                return Err(short_error(e));
                            }
                        };
                    }
                }
                Err(_) => {
                    return Err(format!("Invalid integer value: {}", value));
                }
            };
        }
        _ => {
            //these are all real values
            match value.parse::<f64>() {
                Ok(value) => match control_id.as_str() {
                    "light_key_compound_recovery" => {
                        if value!=unit.get_light_key_compound_recovery()
                        {
                            match unit.set_light_key_compound_recovery(value) {
                                Ok(()) => {
                                    modified = true;
                                }
                                Err(e) => {
                                    return Err(short_error(e));
                                }
                            }
                        }
                    }
                    "heavy_key_compound_recovery" => {
                        if value!= unit.get_heavy_key_compound_recovery() {
                            match unit.set_heavy_key_compound_recovery(value) {
                                Ok(()) => {
                                    modified = true;
                                }
                                Err(e) => {
                                    return Err(short_error(e));
                                }
                            }
                        }
                    }
                    "reflux_ratio_factor" => {
                        if value != unit.get_reflux_ratio_factor() {
                            match unit.set_reflux_ratio_factor(value) {
                                Ok(()) => {
                                    modified = true;
                                }
                                Err(e) => {
                                    return Err(short_error(e));
                                }
                            }
                        }
                    }
                    "convergence_tolerance" => {
                        if value != unit.get_convergence_tolerance() {
                            match unit.set_convergence_tolerance(value) {
                                Ok(()) => {
                                    modified = true;
                                }
                                Err(e) => {
                                    return Err(short_error(e));
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(format!("Unknown control ID: {}", control_id));
                    }
                },
                Err(_) => {
                    return Err(format!("Invalid numeric value: {}", value));
                }
            };
        }
    }
    Ok(modified)
}


/// Handles one [`InputMessage`] received from the editor process, updating unit
/// state and/or the shared `modified` flag as needed, and pushing the appropriate
/// [`UpdateMessage`] reply/replies back to the editor via `tx`.
///
/// - [`InputMessage::SetChannel`]: completes the IPC handshake by storing the sender
///   the editor gave us into `*tx`.
/// - [`InputMessage::DataEntry`]: applies the edit via [`data_entry`], sets
///   `*modified` on success, and always replies with an updated [`Status`] (an error
///   status if the edit was rejected) followed by the refreshed [`Content`].
/// - [`InputMessage::InitialUpdate`]: sent once by the editor when its UI has
///   loaded; replies with the host `parent` window handle (if any, so the dialog can
///   stay above it), the stream table, the initial content, and the initial status.
///
/// `tx` must already be `Some` by the time a `DataEntry` or `InitialUpdate` message
/// arrives, since `SetChannel` is always the first message sent by the editor.
///
/// # Arguments
/// * `unit` - The unit being edited; may be updated by a `DataEntry` message.
/// * `msg` - The message received from the editor process.
/// * `tx` - The reply channel to the editor.
/// * `modified` - Accumulator set to `true` whenever a `DataEntry` message actually
///   changes a unit parameter; left untouched otherwise.
///
/// # Returns
/// Nothing; effects are applied to `unit`, `tx`, and `modified` in place, and
/// replies are pushed to the editor over `tx`.
fn handle_input_message(unit: &mut DistillationShortcutUnit, msg: InputMessage, tx: &IpcSender<UpdateMessage>, modified: &mut bool) {
    match msg {
        InputMessage::DataEntry(entry) => {
            match data_entry(unit, &entry) {
                Ok(modified_entry) => {
                    if modified_entry {
                        *modified = true;
                    }
                    //send status
                    tx.send(UpdateMessage::UpdateStatus { status: get_status(unit) }).unwrap();
                }
                Err(e) => {
                    let status = Status { text: e, error: true };
                    let update_msg = UpdateMessage::UpdateStatus { status };
                    let _ = tx.send(update_msg);
                }
            }
            //send content
            tx.send(UpdateMessage::UpdateContent { content: get_content(unit) }).unwrap();
        },
        InputMessage::InitialUpdate => {
            //send streams
            let _=tx.send(UpdateMessage::UpdateStreams { streams: get_streams(unit) });
            //send content
            let _=tx.send(UpdateMessage::UpdateContent { content: get_content(unit) });
            //send status
            let _=tx.send(UpdateMessage::UpdateStatus { status: get_status(unit) });
        },
        _ => {
            //unexpected, assert when running in debug mode
            #[cfg(debug_assertions)]
            panic!("Unexpected message received from editor");
        }

    }
}

/// Implements the CAPE-OPEN `Edit()` operation: launches the out-of-process editor
/// dialog for `unit`, blocks until the user closes it, and reports whether any
/// parameter was changed.
///
/// `parent` identifies the flowsheet host's window (if any), which is forwarded to
/// the editor so its dialog can be positioned/kept above it; see
/// [`InputMessage::InitialUpdate`] handling in [`handle_input_message`].
///
/// Steps:
/// 1. Locates `distillation_shortcut_unit_editor.exe` next to the current module's
///    own binary/library.
/// 2. Creates a one-shot IPC server and spawns the editor process, passing the
///    server's name as its sole command-line argument.
/// 3. Accepts the editor's connection and processes its first message (the
///    `SetChannel` handshake).
/// 4. Runs a receive loop, handling each [`InputMessage`] via
///    [`handle_input_message`], until the connection closes (the editor process
///    exits when its dialog is closed).
///
/// On Windows, if `parent` has a window handle, that window is temporarily
/// re-enabled while the modal-like editor is up (mirroring how a true modal dialog
/// would behave), and if the parent window belongs to the calling thread, a Win32
/// message pump is run alongside a non-blocking receive loop so the host UI
/// remains responsive; otherwise messages are received with a blocking `recv`.
///
/// # Arguments
/// * `unit` - The unit to edit; mutated in place as the user changes parameters.
/// * `parent` - The flowsheet host's window handle (if any), used to position the
///   editor dialog above it and, on Windows, to pump host messages while the
///   editor is open on the same thread.
///
/// # Returns
/// * `Ok(`[`cape_open_1_2::CapeEditResult::CapeModified`]`)` - The user changed at
///   least one parameter before closing the dialog.
/// * `Ok(`[`cape_open_1_2::CapeEditResult::CapeNotModified`]`)` - The dialog was
///   closed without changing any parameter.
/// * `Err` - The editor executable could not be located next to this module's own
///   binary, or the editor process failed to launch.
pub fn edit(unit: &mut DistillationShortcutUnit, parent: CapeWindowId) -> Result<cape_open_1_2::CapeEditResult, Box<dyn std::error::Error>> {
    //find the out-of-process editor
    let mut editor_path = match process_path::get_dylib_path() {
        Some(path) => path,
        None => return Err("Unable to locate editor path".into()),
    };
    #[cfg(target_os = "windows")]
    {
        editor_path=editor_path.parent().unwrap().join("distillation_shortcut_unit_editor.exe");
    }
    #[cfg(not(target_os = "windows"))]
    {
        editor_path=editor_path.parent().unwrap().join("distillation_shortcut_unit_editor");
    }
    //create a one shot IPC server
    let (server, server_name) = IpcOneShotServer::<InputMessage>::new().unwrap();
    //launch the editor process, passing the server name as arguments
    let mut command = std::process::Command::new(editor_path);
    command.arg(server_name);
    let _child =match command.spawn() {
        Ok(child) => child,
        Err(e) => return Err(format!("Failed to launch editor process: {}", e).into()),
    };
    //accept the connection from the editor process
    let (rx,msg)=server.accept()?;
    let mut modified=false;
    //the first messsage we receive is the transmit channel
    let tx=match msg {
        InputMessage::SetChannel{channel} => {
            channel
        },
        _ => {
            return Err("Expected SetChannel message from editor".into());
        }
    };
    //the first message we send is the parent window
    let mut parent_wnd : ParentWindow = Default::default();
    #[cfg(target_os = "windows")]
    if parent.h_wnd != std::ptr::null_mut() {
        parent_wnd.windows_hwnd = Some(parent.h_wnd as usize as u32);
    }
    #[cfg(not(target_os = "windows"))]
    match parent {
        CapeWindowId::CapeWindow_X11{display_string, xid} => {
            if (!display_string.is_empty()) && (xid!=0) {
                parent_wnd.x11_display = Some(display_string.clone());
                parent_wnd.x11_window_id = Some(xid);
            }
        },
        CapeWindowId::CapeWindow_Wayland{handle, protocol} => {
            if !handle.is_empty() {
                parent_wnd.wayland_external_id = Some(handle.clone());
                if protocol == cobia::C::eWaylandForeignProtocol_XDG_foreign_v1 {
                    parent_wnd.wayland_foreign_protocol_version = Some(1);
                } else if protocol == cobia::C::eWaylandForeignProtocol_XDG_foreign_v2 {
                    parent_wnd.wayland_foreign_protocol_version = Some(2);
                }
            }
        },
        _ => {}
    };
    let _=tx.send(UpdateMessage::ParentWindow{parent_window:parent_wnd});
    //For Windows, we enable/disable the parent if there is one, and in addition if the parent is in the current thread, we pump messages while waiting for the editor to finish.
    #[cfg(target_os = "windows")]  //must include message pump
    {
        use windows::Win32::{
            System::Threading::{GetCurrentThreadId},
            UI::{WindowsAndMessaging::{SetForegroundWindow,PeekMessageW,TranslateMessage,DispatchMessageW,GetWindowThreadProcessId,MSG,PM_REMOVE},
            Input::KeyboardAndMouse::EnableWindow}
        };
        let mut do_process_messages=false;
        let parent_hwnd = if parent.h_wnd == std::ptr::null_mut() {
            None
        } else {
            //check if the parent window is in this thread - if so, we need to pump messages
            let hwnd=windows::Win32::Foundation::HWND{0:parent.h_wnd as *mut _};
            let thread_id= unsafe { GetWindowThreadProcessId(hwnd, None) };
            let current_thread_id=unsafe { GetCurrentThreadId() };
            do_process_messages= thread_id==current_thread_id;
            Some(hwnd)
        };
        if let Some(hwnd) = parent_hwnd {
            unsafe {
                let _ = EnableWindow(hwnd, true);
            }
        }
        if do_process_messages {
            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        handle_input_message(unit, msg, &tx, &mut modified);
                    },
                    Err(TryRecvError::IpcError(_)) => {
                        break;
                    },
                    _ => {}
                }
                //pump messages
                let mut msg = MSG {..Default::default()};
                unsafe {
                    while PeekMessageW(&mut msg, None, 0, 0x002C, PM_REMOVE).as_bool() {
                        let _=TranslateMessage(&msg);
                        let _=DispatchMessageW(&msg);
                    }
                }
            }
            if let Some(hwnd) = parent_hwnd {
                unsafe {
                    let _ = EnableWindow(hwnd, true);
                    let _ = SetForegroundWindow(hwnd);
                }
            }
        } else {
            //no message pump needed, just receive messages
            while let Ok(msg) = rx.recv() {
                handle_input_message(unit, msg, &tx, &mut modified);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]  //no  message pump
    while let Ok(msg) = rx.recv() {
        handle_input_message(unit, msg, &tx, &mut modified);
    }

    Ok(if modified {cape_open_1_2::CapeEditResult::CapeModified} else {cape_open_1_2::CapeEditResult::CapeNotModified})
}

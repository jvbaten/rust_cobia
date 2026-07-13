mod dimensionality;
mod information_stream;
mod material_stream;
mod number_formatter;
mod persistence;
mod property_package_instance;
mod real_parameter;
mod simulation_context;
mod status;
mod status_notification;
mod stream;
mod stream_specification;
mod table_status;
mod thermo_status;
mod unit;
mod unit_status;
mod xml_reader_node;
mod window;

mod unit_operation_runner_data;
use cobia::*;
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};
use status_notification::StatusNotification;
use std::sync::Mutex;
use table_status::TableStatus;
use tauri::{
    AppHandle, Builder, Emitter, Manager, State,
    image::Image,
    menu::{IconMenuItem, MenuBuilder, SubmenuBuilder},
};
use tauri_plugin_dialog::{DialogExt, FilePath, MessageDialogButtons, MessageDialogKind};
use unit_operation_runner_data::UnitOperationRunnerData;
use crate::{thermo_status::ThermoStatus, unit_status::UnitStatus};

/// Emits frontend update events for all pending status notifications.
///
/// Notifications are consumed from the state and translated into specific
/// `update-*` events with their current payloads.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state containing pending notifications.
fn send_status(app: AppHandle, state: &mut UnitOperationRunnerData) {
    let notifications = state.clear_status_notifications();
    if notifications != StatusNotification::None {
        if notifications.contains(StatusNotification::ThermoChanged) {
            app.emit("update-thermo", state.get_thermo_status())
                .unwrap();
        }
        if notifications.contains(StatusNotification::UnitChanged) {
            app.emit("update-unit", state.get_unit_status()).unwrap();
        }
        if notifications.contains(StatusNotification::StreamsChanged) {
            app.emit("update-streams", state.get_stream_table())
                .unwrap();
        }
        if notifications.contains(StatusNotification::ParametersChanged) {
            app.emit("update-parameters", state.get_parameter_table())
                .unwrap();
        }
        if notifications.contains(StatusNotification::ReportsChanged) {
            app.emit("update-reports", state.get_report_list()).unwrap();
        }
        if notifications.contains(StatusNotification::InfoChanged) {
            app.emit("update-info", state.get_info_table()).unwrap();
        }
        if notifications.contains(StatusNotification::LogChanged) {
            app.emit("update-log", state.get_log_table()).unwrap();
        }
        if notifications.contains(StatusNotification::StatusChanged) {
            app.emit("update-status", state.get_status()).unwrap();
        }
    }
}

/// Returns the current thermodynamic package selection/status.
///
/// # Parameters
/// - `state`: The reference to the runner state containing the thermodynamic status.
///
/// # Returns
/// The current thermodynamic status, including provider, package, and any error messages.
#[tauri::command]
fn get_thermo_status(state: State<'_, Mutex<UnitOperationRunnerData>>) -> ThermoStatus {
    let state = state.lock().unwrap();
    state.get_thermo_status()
}

/// Clears the current thermodynamic setup and emits dependent status updates.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to clear the thermodynamic setup.
#[tauri::command]
fn clear_thermo(app: AppHandle, state: State<'_, Mutex<UnitOperationRunnerData>>) {
    let mut state = state.lock().unwrap();
    state.clear_thermo();
    send_status(app, &mut state);
}


/// Opens the CAPE thermodynamic editor dialog.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to edit the thermodynamic setup.
/// - `window`: The Tauri window instance used to obtain the native handle for dialog parenting.
///
/// # Returns
/// `Some(message)` if the editor call fails, otherwise `None`.
#[tauri::command]
fn edit_thermo(
    app: AppHandle,
    state: State<'_, Mutex<UnitOperationRunnerData>>,
    window: tauri::Window,
) -> Option<String> {
    let mut state = state.lock().unwrap();
    window.set_enabled(false).unwrap();
    let window_id = state.get_window(window.window_handle(), window.display_handle());
    let res = match state.edit_thermo(window_id.cape_window_id()) {
        Ok(_) => None,
        Err(e) => Some(e.to_string()),
    };
    window.set_enabled(true).unwrap();
    state.thermo_changed();
    send_status(app, &mut state);
    res
}

/// Selects a thermodynamic provider and emits status updates.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to update the thermodynamic provider.
/// - `provider`: The identifier of the thermodynamic provider to select.
///
/// # Returns
/// None if the provider was successfully selected, otherwise `Some(message)` with an error description.
#[tauri::command]
fn select_thermo_provider(
    app: AppHandle,
    state: State<'_, Mutex<UnitOperationRunnerData>>,
    provider: &str,
) {
    let mut state = state.lock().unwrap();
    state.select_thermo_provider(provider);
    send_status(app, &mut state);
}

/// Selects a thermodynamic property package and emits status updates.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to update the thermodynamic package.
/// - `package`: The identifier of the thermodynamic package to select.
///
/// # Returns
/// None if the package was successfully selected, otherwise `Some(message)` with an error description.
#[tauri::command]
fn select_thermo_package(
    app: AppHandle,
    state: State<'_, Mutex<UnitOperationRunnerData>>,
    package: &str,
) {
    let mut state = state.lock().unwrap();
    state.select_thermo_package(package);
    send_status(app, &mut state);
}

/// Returns the current unit operation selection/status.
///
/// # Parameters
/// - `state`: The reference to the runner state containing the unit operation status.
///
/// # Returns
/// The current unit operation status.
#[tauri::command]
fn get_unit_status(state: State<'_, Mutex<UnitOperationRunnerData>>) -> UnitStatus {
    let state = state.lock().unwrap();
    state.get_unit_status()
}

/// Clears the current unit operation and emits dependent status updates.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to clear the unit operation.
#[tauri::command]
fn clear_unit(app: AppHandle, state: State<'_, Mutex<UnitOperationRunnerData>>) {
    let mut state = state.lock().unwrap();
    state.clear_unit();
    send_status(app, &mut state);
}

/// Opens the CAPE unit-operation editor dialog.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to edit the unit operation.
/// - `window`: The Tauri window instance used to obtain the native handle for dialog parenting.
///
/// # Returns
/// `Some(message)` if the editor call fails, otherwise `None`.
#[tauri::command]
fn edit_unit(
    app: AppHandle,
    state: State<'_, Mutex<UnitOperationRunnerData>>,
    window: tauri::Window,
) -> Option<String> {
    let mut state = state.lock().unwrap();
    window.set_enabled(false).unwrap();
    let window_id = state.get_window(window.window_handle(),window.display_handle());
    let res = match state.edit_unit(window_id.cape_window_id()) {
        Ok(_) => None,
        Err(e) => Some(e.to_string()),
    };
    window.set_enabled(true).unwrap();
    send_status(app, &mut state);
    res
}

/// Selects a unit operation from the discovered unit list.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to update the unit operation selection.
/// - `unit_selection`: The identifier of the unit operation to select.
#[tauri::command]
fn select_unit(
    app: AppHandle,
    state: State<'_, Mutex<UnitOperationRunnerData>>,
    unit_selection: &str,
) {
    let mut state = state.lock().unwrap();
    state.select_unit(unit_selection);
    send_status(app, &mut state);
}

/// Returns overall runner status.
///
/// # Parameters
/// - `state`: The reference to the runner state containing the overall status.
///
/// # Returns
/// The current overall runner status.
#[tauri::command]
fn get_status(state: State<'_, Mutex<UnitOperationRunnerData>>) -> status::Status {
    let state = state.lock().unwrap();
    state.get_status()
}

/// Returns the stream table view model for the UI.
///
/// # Parameters
/// - `state`: The reference to the runner state containing the stream table data.
///
/// # Returns
/// The current stream table view model.
#[tauri::command]
fn get_stream_table(state: State<'_, Mutex<UnitOperationRunnerData>>) -> TableStatus {
    let state = state.lock().unwrap();
    state.get_stream_table()
}

/// Returns the parameter table view model for the UI.
///
/// # Parameters
/// - `state`: The reference to the runner state containing the parameter table data.
///
/// # Returns
/// The current parameter table view model.
#[tauri::command]
fn get_parameter_table(state: State<'_, Mutex<UnitOperationRunnerData>>) -> TableStatus {
    let state = state.lock().unwrap();
    state.get_parameter_table()
}

/// Returns the information-stream table view model for the UI.
///
/// # Parameters
/// - `state`: The reference to the runner state containing the information stream table data.
///
/// # Returns
/// The current information stream table view model.
#[tauri::command]
fn get_info_table(state: State<'_, Mutex<UnitOperationRunnerData>>) -> TableStatus {
    let state = state.lock().unwrap();
    state.get_info_table()
}

/// Applies a single specification edit and emits resulting status updates.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to apply the specification edit.
/// - `edit_id`: The identifier of the specification being edited, used to locate the target parameter and specification type.
/// - `value`: The new value for the specification; empty or whitespace to remove an existing specification.
///
#[tauri::command]
fn specify(
    app: AppHandle,
    state: State<'_, Mutex<UnitOperationRunnerData>>,
    edit_id: &str,
    value: &str,
) {
    let mut state = state.lock().unwrap();
    state.specify(edit_id, value);
    send_status(app, &mut state);
}

/// Executes the selected unit operation and emits resulting status updates.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to execute the unit operation.
#[tauri::command]
fn calculate_unit(app: AppHandle, state: State<'_, Mutex<UnitOperationRunnerData>>) {
    let mut state = state.lock().unwrap();
    state.calculate_unit();
    send_status(app, &mut state);
}

/// Returns the requested report content, sized for the target image frame.
///
/// # Parameters
/// - `state`: The reference to the runner state containing the report content data.
/// - `report`: The identifier of the report to retrieve content for.
/// - `image_width`: The width of the target image frame for rendering the report content.
/// - `image_height`: The height of the target image frame for rendering the report content.
///
/// # Returns
/// A string representation of the report content, formatted for rendering in the frontend. The content may include text and/or encoded image data depending on the report type and current state.
#[tauri::command]
fn get_report_content(
    state: State<'_, Mutex<UnitOperationRunnerData>>,
    report: &str,
    image_width: i32,
    image_height: i32,
) -> String {
    let state = state.lock().unwrap();
    state.get_report_content(report, (image_width, image_height))
}

/// Emits an update event for all status components, typically used after operations that may affect multiple aspects of the state.
/// 
/// This command can be invoked from the frontend to request a full status refresh, for example after batch operations or when 
/// the frontend needs to ensure it has the latest state after a series of edits.
/// 
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to read the latest status and emit updates.
#[tauri::command]
fn request_update(app: AppHandle, state: State<'_, Mutex<UnitOperationRunnerData>>) {
    let mut state = state.lock().unwrap();
    send_status(app, &mut state);
}


/// Saves current runner state to a concrete path and reports errors via dialog.
///
/// # Parameters
/// - `app`: The Tauri application handle used for emitting events.
/// - `state`: The mutable reference to the runner state to save.
/// - `path`: The path to save the runner state to.
fn save_state(app: AppHandle, state: &mut UnitOperationRunnerData, path: std::path::PathBuf) {
    match state.save(path) {
        Ok(_) => {}
        Err(e) => {
            app
                .dialog()
                .message(format!("Failed to save state: {}", e))
                .title("Save State")
                .buttons(MessageDialogButtons::Ok)
                .kind(MessageDialogKind::Error)
                .show(|_ok| {});
        }
    }
}

/// Enables or disables the main webview window during modal file operations.
///
/// # Parameters
/// - `app`: The Tauri application handle used to access the main window.
/// - `enable`: A boolean indicating whether to enable (`true`) or disable (`false`) the main window.
fn enable_window(app: &AppHandle, enable: bool) {
    if let Some(window) = app.get_webview_window("main") {
        window.set_enabled(enable).unwrap();
    }
}

/// Initiates Save or Save As behavior.
///
/// When no path is known or `save_as` is `true`, a save dialog is shown and the
/// write is dispatched onto the main thread to respect COM/PMC threading
/// constraints used by the backend objects.
///
/// # Parameters
/// - `save_as`: A boolean indicating whether to always show the Save As dialog (`true`) or only when no known path exists (`false`).
/// - `app`: The Tauri application handle used for emitting events and showing dialogs.
/// - `state`: The mutable reference to the runner state to save.
fn save(save_as: bool, app: AppHandle, state: &mut UnitOperationRunnerData) {
    let known_path = state.get_path();
    if !save_as && let Some(path) = known_path {
        save_state(app, state, path);
    } else {
        //prompt user for path
        let app1 = app.clone();
        enable_window(&app, false);
        app.dialog()
            .file()
            .add_filter("Unit Runner files", &["unit_runner"])
            .set_title("Save as Unit Operation Runner file")
            .save_file(move |path| match path {
                Some(p) => {
                    match p {
                        FilePath::Path(p) => {
                            let app3 = app1.clone();
                            match app1.run_on_main_thread(move || {
                                let state: State<'_, Mutex<UnitOperationRunnerData>> = app3.state();
                                let mut state = state.lock().unwrap();
                                save_state(app3.clone(), &mut state, p);
                                enable_window(&app3, true);
                            }) {
                                Ok(_) => {}
                                Err(e) => {
                                    let app2 = app1.clone();
                                    app1.dialog()
                                        .message(format!("Failed to save state: {}", e))
                                        .title("Save State")
                                        .buttons(MessageDialogButtons::Ok)
                                        .kind(MessageDialogKind::Error)
                                        .show(move |_ok| {
                                            enable_window(&app2, true);
                                        });
                                }
                            };
                        }
                        FilePath::Url(_) => {
                            app1
                                .dialog()
                                .message("Cannot save to a URL. Please select a file path.")
                                .title("Save as Unit Operation Runner file")
                                .buttons(MessageDialogButtons::Ok)
                                .kind(MessageDialogKind::Error)
                                .show(|_ok| {});
                            enable_window(&app, true);
                        }
                    };
                }
                None => {
                    enable_window(&app, true);
                }
            });
    };
}


/// Run a file without starting the Tauri backend.
///
/// If the file fails to load, a panic occurs with the error message. 
/// 
/// After completing running, the file is saved. If running fails, the error is reported in the file.
/// 
/// If the file fails to save, a panic occurs with the error message.
/// 
/// # Parameters
/// - `filename`: The path to the file to run.
pub fn run_file(filename: String) {
    cape_open_initialize().expect("Failed to initialize COBIA");    
    let mut data = UnitOperationRunnerData::new();
    match data.load(std::path::PathBuf::from(filename.clone())) {
        Ok(_) => {
            //set up streams
            data.update_streams();
            //validate
            match data.get_status() {
                status::Status::Ready => {
                    //calculate
                    data.calculate_unit();
                },
                _ => {}
            }
            //save
            match data.save(std::path::PathBuf::from(filename.clone())) {
                Ok(_) => {},
                Err(e) => {
                    panic!("Failed to save '{}': {}", filename, e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to load '{}': {}", filename, e);
        }
    }
}

/// Starts the Tauri backend, registers commands/menus, and runs the app loop.
///
/// On exit request the internal CAPE/COBIA state is explicitly cleared before
/// process termination to ensure external objects are released predictably.
///
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(filename: Option<String>) {
    //start the application
    let app=Builder::default()
        .setup(|app| {
            //initialize COBIA
            cape_open_initialize().expect("Failed to initialize COBIA");
            //setup application state
            app.manage(Mutex::new(UnitOperationRunnerData::new()));
            //set up menus
            let open_menu = IconMenuItem::with_id(
                app,
                "open",
                "&Open",
                true,
                Some(Image::from_bytes(include_bytes!("../icons/open.png")).unwrap()),
                Some("CmdOrCtrl+O"),
            ).unwrap();
            let save_menu = IconMenuItem::with_id(
                app,
                "save",
                "&Save",
                true,
                Some(Image::from_bytes(include_bytes!("../icons/save.png")).unwrap()),
                Some("CmdOrCtrl+S"),
            ).unwrap();
            let saveas_menu = IconMenuItem::with_id(
                app,
                "saveas",
                "Save &As",
                true,
                Some(Image::from_bytes(include_bytes!("../icons/saveas.png")).unwrap()),
                Some("CmdOrCtrl+Shift+S"),
            ).unwrap();
            let exit_menu = IconMenuItem::with_id(
                app,
                "exit",
                "E&xit",
                true,
                Some(Image::from_bytes(include_bytes!("../icons/exit.png")).unwrap()),
                Some("Alt+F4"),
            ).unwrap();
            let file_menu = SubmenuBuilder::new(app, "&File")
                .item(&open_menu)
                .item(&save_menu)
                .item(&saveas_menu)
                .separator()
                .item(&exit_menu)
                .build()?;
            let menu = MenuBuilder::new(app)
                .items(&[&file_menu])
                .build()?;
            app.set_menu(menu)?;
            //menu handlers
            app.on_menu_event(move |app, event| {
                match event.id().0.as_str() {
                    "open" => {
                        enable_window(app, false);
                        let app1=app.clone();
                        app.dialog().file().
                            add_filter("Unit Runner files", &["unit_runner"]).
                            set_title("Select Unit Operation Runner file").
                            pick_file(move |p| {
                                match p {
                                    Some(p) => {
                                        let path=match p {
                                            FilePath::Path(p) => p,
                                            FilePath::Url(_) => {
                                                app1.dialog()
                                                    .message("Cannot open a URL. Please select a file path.")
                                                    .title("Open Unit Operation Runner file")
                                                    .buttons(MessageDialogButtons::Ok)
                                                    .kind(MessageDialogKind::Error)
                                                    .show(move |_ok| {
                                                            enable_window(&app1, true);
                                                        }
                                                    );
                                                return;
                                            }
                                        };
                                        let app3=app1.clone();
                                        let app4=app1.clone();
                                        match app1.run_on_main_thread(move || {
                                            let state: State<'_, Mutex<UnitOperationRunnerData>> = app4.state();
                                            let mut state = state.lock().unwrap();
                                            //run Load on main thread as we contrain all PMCs to restricted threading, so we must acces all PMCs from the main thread only
                                            match state.load(path) {
                                                Ok(_) => {
                                                    enable_window(&app4, true);
                                                    send_status(app4.clone(), &mut state);
                                                },
                                                Err(e) => {
                                                    let app2=app4.clone();
                                                    app4.dialog()
                                                        .message(format!("Failed to load: {}", e))
                                                        .title("Load Unit Operation Runner file")
                                                        .buttons(MessageDialogButtons::Ok)
                                                        .kind(MessageDialogKind::Error)
                                                        .show(move |_ok| {
                                                                enable_window(&app2, true);
                                                            });
                                                }
                                            };
                                        }) {
                                            Ok(_) => {},
                                            Err(e) => {
                                                let app2=app3.clone();
                                                app3.dialog()
                                                    .message(format!("Failed to load: {}", e))
                                                    .title("Load Unit Operation Runner file")
                                                    .buttons(MessageDialogButtons::Ok)
                                                    .kind(MessageDialogKind::Error)
                                                    .show(move |_ok| {
                                                            enable_window(&app2, true);
                                                        });
                                            }
                                        };
                                    },
                                    None => {
                                        enable_window(&app1, true);
                                    }
                                };
                            });
                    },
                    "save" => {
                        let state: State<'_, Mutex<UnitOperationRunnerData>> = app.state();
                        let mut state = state.lock().unwrap();
                        save(false, app.clone(), &mut state);
                    },
                    "saveas" => {
                        let state: State<'_, Mutex<UnitOperationRunnerData>> = app.state();
                        let mut state = state.lock().unwrap();
                        save(true, app.clone(), &mut state);
                    },
                    "exit" => {
                        let state: State<'_, Mutex<UnitOperationRunnerData>> = app.state();
                        let mut state = state.lock().unwrap();
                        state.clear();
                        app.exit(0);
                    }
                    _ => {}
               }
            });
            if let Some(filename) = filename {
                //if a file name is provided, open it
                let state: State<'_, Mutex<UnitOperationRunnerData>> = app.state();
                let mut state = state.lock().unwrap();
                match state.load(std::path::PathBuf::from(filename.clone())) {
                    Ok(_) => {},
                    Err(e) => {
                        app.dialog()
                            .message(format!("Failed to load '{}': {}", filename, e))
                            .title("Load Unit Operation Runner file")
                            .buttons(MessageDialogButtons::Ok)
                            .kind(MessageDialogKind::Error)
                            .show(move |_ok| {
                                    //do nothing special on close
                            });
                    }
                };
            }
            //done
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
                get_thermo_status,
                clear_thermo,
                edit_thermo,
                select_thermo_provider,
                select_thermo_package,
                get_unit_status,
                clear_unit,
                edit_unit,
                select_unit,
                get_status,
                get_stream_table,
                get_parameter_table,
                get_info_table,
                specify,
                calculate_unit,
                get_report_content,
                request_update
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    //Tauri's App.run() does not return. It will in fact call std::process::exit()
    // which in turn will make that Drop is not called on any of the objects, in
    // particular the UnitOperationRunnerData object that holds the COBIA state.
    // To ensure that the COBIA objects are properly dropped we do this upon
    // the ExitRequested event.
    app.run(|app_handle, event| if let tauri::RunEvent::ExitRequested { .. } = event {
        let state: State<'_, Mutex<UnitOperationRunnerData>> = app_handle.state();
        state.lock().unwrap().clear();
    });
    //run does not exit
    unreachable!();
}

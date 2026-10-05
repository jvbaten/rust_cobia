//! # IPC message types shared between the unit and its out-of-process editor
//!
//! The distillation shortcut unit runs its editor UI in a separate process (a Tauri
//! app), because the UI toolkit cannot run reliably inside a foreign thread hosted by
//! the CAPE-OPEN flowsheeting environment. The unit ("server") and the editor process
//! ("client") communicate over an `ipc_channel` connection using the message types
//! defined here.
//!
//! This module is used in both the `distillation_shortcut_unit` crate
//! (server side) and the `distillation_shortcut_unit_editor` crate (client side), so
//! it is stored in this shared folder.
//!
//! The initial handshake works as follows: the server starts the editor process,
//! passing the name of an [`IpcOneShotServer`](ipc_channel::ipc::IpcOneShotServer) on
//! the command line. The editor connects and sends [`InputMessage::SetChannel`],
//! handing the server a sender it can use to push [`UpdateMessage`]s back. From then
//! on, [`UpdateMessage`] flows server -> editor (to render the dialog) and
//! [`InputMessage`] flows editor -> server (to report user input).

use serde::{Serialize,Deserialize};
use ipc_channel::ipc::IpcSender;

/// Snapshot of the unit's editable parameters, sent to the editor to populate the dialog.
#[derive(Serialize,Deserialize,Clone)]
pub struct Content {
    /// The name of the unit operation, as shown in the flowsheet.
    pub unit_name: String,
    /// The names of the compounds in the feed material object, used to populate the
    /// light/heavy key compound selection controls.
    pub compound_list: Vec<String>,
    /// The description of the unit operation.
    pub unit_description: String,
    /// The currently selected light key compound.
    pub light_key_compound: String,
    /// The currently selected heavy key compound.
    pub heavy_key_compound: String,
    /// The specified recovery (fraction) of the light key compound.
    pub light_key_compound_recovery: f64,
    /// The specified recovery (fraction) of the heavy key compound.
    pub heavy_key_compound_recovery: f64,
    /// The specified reflux ratio, as a factor above the minimum reflux ratio.
    pub reflux_ratio_factor: f64,
    /// The maximum number of iterations allowed for the shortcut calculation.
    pub maximum_iterations: i32,
    /// The convergence tolerance for the shortcut calculation.
    pub convergence_tolerance: f64,
    /// The calculated number of theoretical stages (calculation result).
    pub number_of_stages: f64,
    /// The calculated reflux ratio (calculation result).
    pub reflux_ratio: f64,
    /// The calculated feed stage location (calculation result).
    pub feed_stage_location: f64,
}

/// Status line shown in the editor dialog (e.g. validation or calculation result).
#[derive(Serialize,Deserialize,Clone)]
pub struct Status {
    /// The message text to display.
    pub text: String,
    /// Whether the message represents an error (affects how it is styled/presented).
    pub error: bool,
}

/// Native handle of the flowsheet host's window, used by the editor so it can keep
/// its dialog on top of the window that owns it. Exactly one field is populated,
/// depending on the platform/windowing system the host is running under.
#[derive(Serialize,Deserialize,Default)]
pub struct ParentWindow {
    /// The host window handle (`HWND`), on Windows.
    pub windows_hwnd : Option<u32>,
    /// The X11 display name, when running under X11.
    pub x11_display: Option<String>,
    /// The X11 window ID of the host window, when running under X11.
    pub x11_window_id: Option<u64>,
    /// The external window handle/identifier, when running under Wayland.
    pub wayland_external_id: Option<String>,
    /// The version (1 or 2) of the XDG foreign protocol under which
    /// `wayland_external_id` was exported, when running under Wayland.
    pub wayland_foreign_protocol_version: Option<u32>,
}

/// Messages sent from the unit (server) to the editor process (client).
#[derive(Serialize,Deserialize)]
pub enum UpdateMessage {
    /// Update the status line shown in the dialog.
    UpdateStatus{
        /// The new status to display.
        status: Status
    },
    /// Replace the dialog's editable parameter values.
    UpdateContent{
        /// The new parameter values to display.
        content: Content
    },
    /// Replace the table of connected streams shown in the dialog.
    UpdateStreams{
        /// One row per connected port; each row is a list of display strings
        /// (e.g. port name and connected stream name) for that port.
        streams: Vec<Vec<String>>
    },
    /// Report the host window the editor dialog should stay on top of.
    ParentWindow{
        /// The host window handle to track.
        parent_window: ParentWindow
    },
}


/// A single control's value as entered by the user in the editor dialog.
#[derive(Serialize,Deserialize)]
pub struct DataEntry {
    /// Identifier of the control that was edited (matches an ID in the dialog's HTML).
    pub control_id: String,
    /// The new value entered for the control, as text.
    pub value: String,
}

/// Messages sent from the editor process (client) to the unit (server).
#[derive(Serialize,Deserialize)]
pub enum InputMessage {
    /// The user changed the value of a control in the dialog.
    DataEntry(DataEntry),
    /// The editor UI has loaded and is ready to receive its initial [`Content`].
    InitialUpdate,
    /// Handshake message: hands the server a sender it can use to push
    /// [`UpdateMessage`]s to this editor instance.
    SetChannel{
        /// Sender the server should use to push [`UpdateMessage`]s to this editor.
        channel: IpcSender<UpdateMessage>
    },
}


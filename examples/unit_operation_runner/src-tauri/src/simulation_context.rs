use std::sync::LazyLock;

use cobia::*;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

/// Interned CAPE string constants used by [`SimulationContext`].
///
/// Centralises the `ICapeCOSEUtilities` named-value keys so they are allocated
/// once and compared without repeated heap allocations.
pub(crate) struct SimulationContextConstants {
    /// Key for the report rendering image width, in pixels.
    image_width: CapeStringConstNoCase,
    /// Key for the report rendering image height, in pixels.
    image_height: CapeStringConstNoCase,
    /// Key for the HTML report support flag.
    html_report_support: CapeStringConstNoCase,
    /// Key for the character encoding descriptor.
    character_encoding: CapeStringConstNoCase,
}

impl SimulationContextConstants {
    /// Constructs string constants.
    ///
    /// # Returns
    /// A fully initialised [`SimulationContextConstants`].
    fn new() -> Self {
        Self {
            image_width: CapeStringConstNoCase::from_string("ImageWidth"),
            image_height: CapeStringConstNoCase::from_string("ImageHeight"),
            html_report_support: CapeStringConstNoCase::from_string("HTMLReportSupport"),
            character_encoding: CapeStringConstNoCase::from_string("CharacterEncoding"),
        }
    }
}

static SIMULATION_CONTEXT_CONSTANTS: LazyLock<SimulationContextConstants> =
    LazyLock::new(SimulationContextConstants::new);

/// CAPE-OPEN simulation context provided to unit operations.
///
/// Implements `ICapeIdentification`, `ICapeDiagnostic`,
/// `ICapeCOSEUtilities`, and `ICapeSimulationContext`.
///
/// Diagnostic messages are collected internally via `LogMessage` and can be
/// retrieved by the host with `take_messages`. Pop-up dialogs are forwarded
/// to the Tauri window via the stored [`AppHandle`].
#[cape_object_implementation(
		interfaces = {
			cape_open_1_2::ICapeIdentification,
			cape_open_1_2::ICapeDiagnostic,
			cape_open_1_2::ICapeCOSEUtilities,
            cape_open_1_2::ICapeSimulationContext,
		},
        create_arguments = {},
        default_values = {image_width = 800, image_height = 600},
  )]
pub(crate) struct SimulationContext {
    /// Log messages accumulated via `ICapeDiagnostic::LogMessage`.
    messages: Vec<String>,
    /// Tauri application handle used to show pop-up dialogs. `None` before the application window is available.
    app: Option<AppHandle>,
    /// Width in pixels reported to the unit operation for report image rendering.
    image_width: i32,
    /// Height in pixels reported to the unit operation for report image rendering.
    image_height: i32,
}

impl SimulationContext {
    /// Removes and returns all accumulated log messages.
    ///
    /// Leaves the internal message buffer empty.
    ///
    /// # Returns
    /// The collected messages in the order they were logged.
    pub fn take_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.messages)
    }

    /// Discards all accumulated log messages.
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Updates the image resolution reported to unit operations via
    /// `ICapeCOSEUtilities::NamedValue`.
    ///
    /// # Parameters
    /// - `width`: Image width in pixels.
    /// - `height`: Image height in pixels.
    pub fn set_image_resolution(&mut self, width: i32, height: i32) {
        self.image_width = width;
        self.image_height = height;
    }

    /// Enables or disables the main webview window.
    ///
    /// Used to block user interaction while a modal dialog is shown by the
    /// CAPE unit operation.
    ///
    /// # Parameters
    /// - `app`: Tauri application handle.
    /// - `enable`: `true` to enable, `false` to disable.
    fn enable_window(app: &AppHandle, enable: bool) {
        if let Some(window) = app.get_webview_window("main") {
            window.set_enabled(enable).unwrap();
        }
    }
}

impl std::fmt::Display for SimulationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimulationContext")
    }
}

impl cape_open_1_2::ICapeIdentification for SimulationContext {
    fn get_component_name(&mut self, name: &mut CapeStringOut) -> Result<(), COBIAError> {
        name.set_string("Simulation Context")
    }

    fn set_component_name(&mut self, _name: &CapeStringIn) -> Result<(), COBIAError> {
        Err(COBIAError::Code(COBIAERR_DENIED))
    }

    fn get_component_description(&mut self, desc: &mut CapeStringOut) -> Result<(), COBIAError> {
        desc.set_string("Simulation Context for Unit Operation Runner")
    }

    fn set_component_description(&mut self, _desc: &CapeStringIn) -> Result<(), COBIAError> {
        Err(COBIAError::Code(COBIAERR_DENIED))
    }
}

impl cape_open_1_2::ICapeDiagnostic for SimulationContext {
    fn pop_up_message(&mut self, message: &CapeStringIn) -> Result<(), COBIAError> {
        match self.app {
            Some(ref app) => {
                Self::enable_window(app, false);
                let app1 = app.clone();
                app.dialog()
                    .message(message.to_string())
                    .title("")
                    .buttons(MessageDialogButtons::Ok)
                    .kind(MessageDialogKind::Info)
                    .show(move |_ok| {
                        Self::enable_window(&app1, true);
                    });
                Ok(())
            }
            None => Err(COBIAError::Code(COBIAERR_NOTIMPLEMENTED)),
        }
    }

    fn log_message(&mut self, message: &CapeStringIn) -> Result<(), COBIAError> {
        self.messages.push(message.to_string());
        Ok(())
    }
}

impl cape_open_1_2::ICapeCOSEUtilities for SimulationContext {
    fn get_named_value_list(
        &mut self,
        named_values: &mut CapeArrayStringOut,
    ) -> Result<(), COBIAError> {
        named_values.resize(4)?;
        named_values
            .at(0)?
            .set(&SIMULATION_CONTEXT_CONSTANTS.image_width)?;
        named_values
            .at(1)?
            .set(&SIMULATION_CONTEXT_CONSTANTS.image_height)?;
        named_values
            .at(2)?
            .set(&SIMULATION_CONTEXT_CONSTANTS.html_report_support)?;
        named_values
            .at(3)?
            .set(&SIMULATION_CONTEXT_CONSTANTS.character_encoding)?;
        Ok(())
    }

    fn named_value(
        &mut self,
        name: &CapeStringIn,
        named_value: &mut CapeValueOut,
    ) -> Result<(), COBIAError> {
        if &SIMULATION_CONTEXT_CONSTANTS.image_width == name {
            named_value.set_integer(self.image_width)?;
        } else if &SIMULATION_CONTEXT_CONSTANTS.image_height == name {
            named_value.set_integer(self.image_height)?;
        } else if &SIMULATION_CONTEXT_CONSTANTS.html_report_support == name {
            named_value.set_boolean(true)?;
        } else if &SIMULATION_CONTEXT_CONSTANTS.character_encoding == name {
            named_value.set_string("UTF-8")?;
        } else {
            return Err(COBIAError::Code(COBIAERR_NOSUCHITEM));
        }
        Ok(())
    }
}

impl cape_open_1_2::ICapeSimulationContext for SimulationContext {}

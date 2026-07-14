//! # Driver for the distillation shortcut unit editor
//!
//! See [`distillation_shortcut_unit_editor_lib`] for the actual implementation of the unit editor. 
//! 
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]



/// Entry point for the unit editor executable.
///
/// In release builds, installs a panic hook that reports the panic message via a
/// Windows message box (or stderr on other platforms) before exiting, since the
/// process runs without a console (`windows_subsystem = "windows"`) and would
/// otherwise fail silently. Then hands off control to
/// [`distillation_shortcut_unit_editor_lib::show_dialog`], which runs the editor's
/// message loop and does not return.
fn main() {

    #[cfg(not(debug_assertions))]
    {
        //set panic hook to show message box on windows, or print error on other platforms
        std::panic::set_hook(Box::new(|info| {
            let msg = match info.payload_as_str() {
                Some(s) => s.to_string(),
                None => "Unknown error".to_string(),
            };
            #[cfg(target_os = "windows")]
            {
                use windows::{
                    Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK,MB_ICONHAND},
                    core::HSTRING
                };
                unsafe {
                    MessageBoxW(None,&HSTRING::from(msg),&HSTRING::from("Unit Operation Runner"), MB_OK|MB_ICONHAND);
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                eprintln!("Error: {}", msg);
            }
            std::process::exit(-1);
        }));
    }

    distillation_shortcut_unit_editor_lib::show_dialog();
    unreachable!();

}

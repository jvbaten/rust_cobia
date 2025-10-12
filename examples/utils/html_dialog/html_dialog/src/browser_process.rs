//! Process-based browser implementation.
//!
//! This module provides a fallback browser implementation that launches an external browser
//! process (Google Chrome) when a native browser component like WebView2 is not available.
//! 
//! This implementation may also be useful to use a browser for development purposes, allowing
//! access to the web developers tools in the browser.
//! 
//! It implements the `BrowserMonitor` trait to maintain a consistent interface for browser
//! communication within the HTML dialog system.

use std::{
    path::Path,
    process::{Command, Child},
};

#[cfg(target_os = "windows")]
use super::message_loop;

/// Monitors an external browser process.
///
/// This struct implements the `BrowserMonitor` trait for an external browser process,
/// providing a way to track the lifecycle of a browser running as a separate process.
struct ExternalProcessMonitor {
    /// Handle to the spawned child process
    child: Child,
}

impl ExternalProcessMonitor {
    /// Creates a new external process monitor.
    ///
    /// # Parameters
    /// * `child` - Handle to the spawned child process to monitor
    ///
    /// # Returns
    /// A new `ExternalProcessMonitor` instance
    fn new(child: Child) -> Self {
        ExternalProcessMonitor { child }
    }
}

impl super::browser::BrowserMonitor for ExternalProcessMonitor {
    /// Checks if the browser process has terminated.
    ///
    /// # Returns
    /// * `Ok(true)` - If the process has terminated or if the status check failed
    /// * `Ok(false)` - If the process is still running
    fn is_terminated(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(match self.child.try_wait() {
            Ok(Some(_)) => true,    // Process has exited
            Ok(None) => false,      // Process is still running
            Err(_) => true,         // If we can't check, assume it's terminated
        })
    }

    /// Attempts to resize the browser window.
    ///
    /// This operation is not supported for external processes directly.
    /// JavaScript's window.resizeTo should be used instead.
    ///
    /// # Parameters
    /// * `_width` - Desired width (unused)
    /// * `_height` - Desired height (unused)
    ///
    /// # Returns
    /// * `Err` - Always returns an error. Use JavaScript instead.
    fn resize_request(&mut self, _width: u32, _height: u32) -> Result<(), Box<dyn std::error::Error>> {
        Err("resize supported; use window.resizeTo in javascript".into())
    }

    /// Terminates the browser process.
    ///
    /// Attempts to forcibly kill the child process.
    fn terminate(&mut self) {
        let _ = self.child.kill();
    }

    /// Gets the window handle associated with the browser.
    ///
    /// External processes do not provide access to their window handles
    /// through this interface.
    ///
    /// # Returns
    /// * `None` - Always returns None as window access is not supported
    fn get_window(&mut self) -> Option<super::window::Window> {
        None // Not supported for external processes
    }

    /// Processes any pending messages for the browser.
    ///
    /// On Windows, this pumps the message loop to keep the application responsive.
    /// On other platforms, this function has no effect.
    ///
    /// # Returns
    /// * Whether any messages were processed
    fn pump_messages(&mut self) -> bool {
        #[cfg(target_os = "windows")]
        message_loop::pump()
    }
}

/// Starts a browser as an external process.
///
/// This function attempts to locate and launch Google Chrome as an external process
/// to display the given URL. Currently, this is only implemented for Windows platforms
/// and requires Chrome to be installed.
///
/// # Parameters
/// * `url` - The URL to load in the browser
///
/// # Returns
/// * `Ok(Box<dyn BrowserMonitor>)` - A boxed monitor for the launched browser process
/// * `Err(...)` - If browser launch failed.
pub(crate) fn start_browser_as_process(url: &str) -> Result<Box<dyn super::browser::BrowserMonitor>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        // Locate Chrome browser through Windows registry
        match registry::Hive::LocalMachine.open(r"SOFTWARE\\Classes\\ChromeHTML\\shell\\open\\command", registry::Security::Read) {
            Ok(key) => {
                match key.value("") {
                    Ok(command) => {
                        // Parse the command string to extract the executable path
                        let mut command = command.to_string();
                        if command.starts_with('"') {
                            // Handle quoted path
                            command = command[1..].to_string();
                            if let Some(pos) = command.find('"') {
                                command = command[..pos].to_string();
                            }
                        } else {
                            // Handle unquoted path
                            if let Some(pos) = command.find(' ') {
                                command = command[..pos].to_string();
                            }
                        }
                        
                        // Verify the path exists and launch Chrome
                        if Path::new(&command).exists() {
                            // Launch Chrome in app mode with the specified URL
                            let mut arg = String::with_capacity(6 + url.len());
                            arg.push_str("--app=");
                            arg.push_str(url);
                            return Ok(Box::new(ExternalProcessMonitor::new(Command::new(command).arg(arg).spawn()?)));
                        }
                    },
                    Err(_) => {}
                }
            },
            Err(_) => {}
        }
        
        // If we get here, Chrome wasn't found or couldn't be launched
        Err("No suitable browser found. Please make sure Google Chrome is installed".into())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("This function is only implemented for Windows".into())
    }
}
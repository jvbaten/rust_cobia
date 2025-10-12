//! Browser module for the HTML dialog system.
//!
//! Provides a platform-agnostic interface for different browser implementations
//! and handles browser selection based on the platform and available capabilities.

use super::window::Window;

/// Defines the interface for monitoring and controlling a browser instance.
///
/// Implementations of this trait are responsible for managing the lifecycle and
/// interaction with different browser backends (Edge WebView2, external process, etc.).
pub(crate) trait BrowserMonitor {
    /// Checks if the browser has been terminated.
    ///
    /// # Returns
    /// - `Ok(true)` if the browser has been terminated
    /// - `Ok(false)` if the browser is still running
    /// - `Err(...)` if there was an error checking the browser status
    fn is_terminated(&mut self) -> Result<bool, Box<dyn std::error::Error>>;

    /// Requests the browser window to resize to the specified dimensions.
    ///
    /// # Parameters
    /// - `width`: The desired width in pixels
    /// - `height`: The desired height in pixels
    ///
    /// # Returns
    /// - `Ok(())` if the resize operation was successful
    /// - `Err(...)` if the resize operation failed.
    fn resize_request(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>>;

    /// Signals the browser to terminate.
    ///
    /// This method initiates browser termination but doesn't guarantee immediate termination.
    /// Check `is_terminated()` to confirm the browser has actually closed.
    fn terminate(&mut self);

    /// Gets the window handle associated with the browser.
    ///
    /// # Returns
    /// - `Some(Window)` if a window handle is available (typically for WebView2)
    /// - `None` if no window handle is available (typically for out of process browsers)
    fn get_window(&mut self) -> Option<Window>;

    /// Processes any pending messages for the browser.
    ///
    /// This method should be called regularly to handle browser events and messages.
    ///
    /// # Returns
    /// - `true` if any messages were processed
    /// - `false` if no messages were processed
    fn pump_messages(&mut self) -> bool;
}

/// Starts a browser instance with the specified parameters.
///
/// This function selects the appropriate browser implementation based on the platform
/// and available capabilities.
///
/// # Parameters
/// - `parent`: Optional parent window to which the browser should be attached
/// - `as_modal`: Whether the browser should be displayed as a modal dialog 
///   (only applies when `parent` is provided)
/// - `url`: The URL to load in the browser
///
/// # Returns
/// - `Ok(Box<dyn BrowserMonitor>)` with an implementation:
/// - `Err(...)` if browser creation failed.
pub(crate) fn start_browser(parent: Option<Window>, as_modal: bool, url: &str) -> Result<Box<dyn BrowserMonitor>, Box<dyn std::error::Error>> {
    // On Windows, try to use EdgeView2 if it's available
    #[cfg(target_os = "windows")]
    {
        if super::browser_edgeview2::is_webview_installed() {
            return super::browser_edgeview2::start_browser_as_edgeview2(parent, as_modal, url);
        }
    }    
    // Fall back to process-based browser implementation
    super::browser_process::start_browser_as_process(url)
}
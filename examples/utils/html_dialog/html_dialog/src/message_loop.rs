//! Windows message loop implementation for the HTML Dialog system.
//!
//! This module provides functionality to process Windows messages in a non-blocking manner,
//! which is essential for keeping the UI responsive while the HTML Dialog is active.
//! It's only compiled on Windows targets and is used by both browser implementations
//! (EdgeView2 and external process) to handle Windows messages.

use windows::{
    core::*,
    Win32::{
        UI::WindowsAndMessaging::{self, MSG, PM_REMOVE},
    },
};

/// Processes pending Windows messages in a non-blocking manner.
///
/// This function serves as a message pump that processes all pending Windows messages
/// without blocking if no messages are available. It's used by the HTML Dialog system
/// to keep the UI responsive during dialog display.
///
/// # Returns
/// * `true` - some messages were processed
/// * `false` - no messages were processed
pub(crate) fn pump() -> bool {
    // Initialize message structure and activity tracker
    let mut msg = MSG::default();
    let mut any: bool = false;
    // Process all pending messages
    loop {
        // PeekMessageW checks for messages without blocking
        // Returns 0 (FALSE) if no messages are available
        if unsafe { WindowsAndMessaging::PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) } == BOOL(0) {
            break;
        }
        // At least one message was processed
        any = true;
        // TranslateMessage converts virtual-key messages to character messages
        let _ = unsafe { WindowsAndMessaging::TranslateMessage(&msg) };
        // DispatchMessageW sends the message to the window procedure
        unsafe { WindowsAndMessaging::DispatchMessageW(&msg) };    
    };
    // Return whether any messages were processed
    any
}
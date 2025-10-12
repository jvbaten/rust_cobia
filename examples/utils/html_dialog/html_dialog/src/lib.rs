//! # HTML Dialog provider
//!
//! For the purpose of add-in components that have their own 
//! modal GUI, the following requirements are needed on a 
//! dialog
//! - cross platform (Windows, Linux, MacOS)
//! - ability to show a dialog in the current GUI thread
//! - ability to show a dialog in a modal manner (blocking the caller and providing the parent window)
//!
//! This crate aims to provide such a dialog by using a local web server.
//!
//! The web server features multiple stay-alive connections, and the dialog messages are processed, all
//! in the same, main thread. This enables the dialog itself to be running in the GUI thread, and it 
//! enables interaction with foreigh components, such as the add-in component that is being edited,
//! to all be constrained to the same thread.
//!
//! The current implementation only works for Windows. Extension to linux and MacOS 
//! is planned using Webkit or CEF functionality.
//!
//! The web server logs to the log crate, so for diagnostic purposes, the host application
//! may set up logging to see the requests and responses. For example:
//! ```
//! #[cfg(debug_assertions)]
//! {
//!     //write http server log to debug console
//!     win_dbg_logger::init();
//!     //write INFO level, then we we all interaction with the http server
//!     log::set_max_level(log::LevelFilter::Debug);
//! }
//! ```


mod connection;
mod browser;
mod browser_process;
mod window;

#[cfg(target_os = "windows")]
mod message_loop;

pub use window::Window;

#[cfg(target_os = "windows")]
mod browser_edgeview2;

use std::{
    io::ErrorKind,
    net::TcpListener,
    cell::RefCell,
};
use connection::{Connection};
use log::{info, error};

/// Handler trait for providing dynamic content in HTML dialogs.
///
/// Implementors of this trait can provide content in response to specific events,
/// allowing for interactive dialogs with custom behavior.
///
/// # Type Parameters
/// * `E` - The event type that triggers content requests
pub trait HtmlDialogHandler<E> {
    /// Generates content for a specific event in the HTML dialog.
    ///
    /// Called when the dialog needs to serve content for a specific path that
    /// is mapped to an Info event.
    ///
    /// # Parameters
    /// * `event` - Reference to the event that triggered the content request
    /// * `content` - Optional content sent with the request (e.g., from a POST request)
    /// * `window` - Optional handle to the browser window, if available
    ///
    /// # Returns
    /// * `Ok((Vec<u8>, String))` - Content as bytes and its MIME type (e.g., "text/html")
    /// * `Err(...)` - If content generation fails
    fn provide_content(&mut self, event: &E, content: Option<String>, window: Option<Window>) 
        -> Result<(Vec<u8>, String), Box<dyn std::error::Error>>;
}

/// Types of resources that can be served by an HTML dialog.
///
/// This enum defines the different kinds of resources that can be mapped to
/// URL paths in the HTML dialog's embedded web server.
///
/// # Type Parameters
/// * `E` - The event type used for Info resources
pub enum HtmlDialogResourceType<E> {
    /// Static content with raw bytes and MIME type
    /// 
    /// # Fields
    /// * `&'static [u8]` - The raw content bytes
    /// * `&'static str` - The MIME type (e.g., "text/html", "application/javascript")
    Content((&'static [u8], &'static str)),
    /// Custom event information to be processed by the HtmlDialogHandler
    Info(E),
    /// Request to resize the browser window
    ResizeRequest,
    /// Request to terminate the dialog
    TerminateRequest,
}

/// Main HTML dialog implementation.
///
/// Provides a web-based dialog using an embedded HTTP server and a browser component.
/// The dialog can be modal or non-modal and can interact with the host application
/// through the HtmlDialogHandler interface.
///
/// # Type Parameters
/// * `T` - The handler type implementing HtmlDialogHandler<E>
/// * `E` - The event enumeration for dialog interactions
pub struct HtmlDialog<T, E> where T: HtmlDialogHandler<E> {
    /// Map of URL paths to resources
    resource_map: std::collections::HashMap<String, HtmlDialogResourceType<E>>,
    /// Content handler for dynamic responses
    handler: T,
    /// Browser instance monitor
    browser_monitor: Option<RefCell<Box<dyn browser::BrowserMonitor>>>,
    /// Optional parent window for modal dialogs
    parent: Option<Window>,
    /// Whether the dialog should be modal (block the parent window)
    as_modal: bool,
}

impl<T, E> HtmlDialog<T, E> where T: HtmlDialogHandler<E> {
    /// Creates a new HTML dialog with the provided content handler.
    ///
    /// Initializes the dialog with standard endpoints for window resizing and
    /// termination requests.
    ///
    /// # Parameters
    /// * `handler` - The implementation of HtmlDialogHandler that will provide content
    ///
    /// # Returns
    /// A new HTML dialog instance
    pub fn new(handler: T) -> Self {
        let mut html_dialog = HtmlDialog {
            resource_map: std::collections::HashMap::new(),
            handler,
            browser_monitor: None,
            parent: None,
            as_modal: true, // Unused if no parent is set
        };
        // Register standard endpoints
        html_dialog.add("/resize_window".into(), HtmlDialogResourceType::ResizeRequest);
        html_dialog.add("/close_window".into(), HtmlDialogResourceType::TerminateRequest);
        // Return the dialog
        html_dialog
    }

    /// Gets the window handle of the browser, if available.
    ///
    /// # Returns
    /// * `Some(Window)` - The browser window handle, if available
    /// * `None` - If no browser is active or the browser doesn't support window handles
    pub fn get_window(&mut self) -> Option<Window> {
        match self.browser_monitor {
            Some(ref b) => (*(b.borrow_mut())).get_window(),
            None => None,
        }
    }

    /// Sets the parent window and modal state for the dialog.
    ///
    /// Must be called before showing the dialog. 
    ///
    /// # Parameters
    /// * `parent` - Optional parent window handle
    /// * `as_modal` - Whether the dialog should be modal (block parent input)
    ///
    /// # Returns true if supported and successful
    pub fn set_parent(&mut self, parent: Option<Window>, as_modal: bool) -> bool {
        if self.browser_monitor.is_some() {
            return false; // Too late, browser already started
        }
        // Store until creation
        self.parent = parent;
        self.as_modal = as_modal;
        true
    }

    /// Maps a URL path to a resource in the dialog.
    ///
    /// # Parameters
    /// * `location` - The URL path to map (e.g., "/index.html")
    /// * `resource` - The resource to serve at this location
    pub fn add(&mut self, location: String, resource: HtmlDialogResourceType<E>) {
        self.resource_map.insert(location, resource);
    }

    /// Shows the dialog and runs its message loop.
    ///
    /// This method starts the embedded HTTP server, launches the browser,
    /// and processes requests until the browser is closed or an error occurs.
    /// It blocks the calling thread until the dialog is closed.
    ///
    /// # Returns
    /// * `Ok(())` - If the dialog was shown and closed successfully
    /// * `Err(...)` - If an error occurred during setup or execution
    pub fn show(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Start HTTP server on a random available port
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        info!("Listening on port: {}", port);
        // Set up parent window reference if provided
        let parent: Option<Window> = match self.parent {
            Some(ref p) => Some(Window{handle: p.handle}),
            None => None,
        };
        // Launch the browser
        let browser_monitor = browser::start_browser(
            parent, 
            self.as_modal, 
            &format!("http://127.0.0.1:{}/", port)
        )?;
        self.browser_monitor = Some(RefCell::new(browser_monitor));
        // Set the listener to non-blocking mode
        match listener.set_nonblocking(true) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to set listener to non-blocking mode: {}", e);
                self.browser_monitor = None;
                return Err(e.into());
            }
        };        
        // Main connection processing loop
        let mut connections = Vec::<Connection>::new();
        for stream in listener.incoming() {
            let mut any_action: bool = false;            
            // Handle new connections
            match stream {
                Ok(s) => {
                    info!("Accepted connection");
                    any_action = true;
                    // Create and add a new connection
                    match Connection::new(s) {
                        Ok(connection) => {
                            connections.push(connection);
                        },
                        Err(e) => {
                            assert!(false, "Failed to create connection: {}", e);
                            continue;
                        }
                    }
                },
                // Handle non-blocking check for browser termination
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    let mut return_code: Option<Result<(), Box<dyn std::error::Error>>> = None;
                    // Check if browser has terminated
                    if let Some(ref b) = self.browser_monitor {
                        match b.borrow_mut().is_terminated() {
                            Ok(terminated) => {
                                if terminated {
                                    info!("Browser process terminated, exiting.");
                                    return_code = Some(Ok(()));
                                }
                            },
                            Err(e) => {
                                error!("Browser error: {}", e);
                                return_code = Some(Err(e));
                            }
                        }
                    }
                    // Return if browser has terminated or errored
                    if let Some(returncode) = return_code {
                        self.browser_monitor = None;
                        return returncode;
                    }
                },
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            };
            // Process all active connections
            connections.retain_mut(|connection| {
                let mut keep = true;
                // Advance connection state
                if connection.advance() {
                    any_action = true;
                }
                // Handle completed or errored connections
                if connection.is_error() || connection.is_complete() {
                    let (content, content_type, response) = if connection.is_error() {
                        // Handle connection errors
                        keep = false;
                        let code = connection.get_error_code();
                        if code != 0 {
                            print!("==> Error: {}\n", connection.get_error());
                        }
                        (connection.get_error().as_bytes().to_vec(), "text/plain".into(), code)
                    } else {
                        // Handle completed requests
                        let parent = self.get_window();
                        let res = match self.resource_map.get(connection.get_location()) {
                            Some(e) => match e {
                                // Serve static content
                                HtmlDialogResourceType::Content((content, content_type)) => 
                                    (content.to_vec(), content_type.to_string(), 200),
                                // Process custom events through handler
                                HtmlDialogResourceType::Info(info) => {
                                    let response_from_content = |content: Vec<u8>| {
                                        // Parse content as UTF-8 if available
                                        let content = if content.is_empty() {
                                            None
                                        } else {
                                            match String::from_utf8(content) {
                                                Ok(c) => Some(c),
                                                Err(_) => {
                                                    return ("invalid post content: invalid UTF-8"
                                                        .as_bytes().to_vec(), "text/plain".into(), 500);
                                                }
                                            }
                                        };
                                        // Call handler to generate response
                                        match self.handler.provide_content(info, content, parent) {
                                            Ok((content, content_type)) => (content, content_type, 200),
                                            Err(e) => (format!("{}", e).as_bytes().to_vec(), 
                                                      "text/plain".into(), 500),
                                        }
                                    };
                                    response_from_content(connection.get_content())
                                },
                                // Handle window resize requests
                                HtmlDialogResourceType::ResizeRequest => {
                                    let content = connection.get_content();
                                    if content.is_empty() {
                                        // No content, error
                                        ("Invalid resize request".as_bytes().to_vec(), 
                                         "text/plain".into(), 500)
                                    } else {
                                        // Parse JSON for width and height
                                        match String::from_utf8(content) {
                                            Ok(c) => {
                                                match json::parse(&c) {
                                                    Ok(j) => {
                                                        let width = j["width"].as_u32().unwrap_or(800);
                                                        let height = j["height"].as_u32().unwrap_or(600);
                                                        // Attempt to resize browser window
                                                        match self.browser_monitor {
                                                            Some(ref b) => {
                                                                match b.borrow_mut().resize_request(width, height) {
                                                                    Ok(_) => ("".as_bytes().to_vec(), 
                                                                             "text/plain".into(), 200),
                                                                    Err(e) => (format!("Failed to resize window: {}", e)
                                                                              .as_bytes().to_vec(), 
                                                                             "text/plain".into(), 500),
                                                                }
                                                            },
                                                            None => {
                                                                ("No browser".as_bytes().to_vec(), 
                                                                 "text/plain".into(), 500)
                                                            }
                                                        }
                                                    },
                                                    Err(e) => (format!("Invalid JSON in resize request: {}", e)
                                                              .as_bytes().to_vec(), 
                                                             "text/plain".into(), 500),
                                                }
                                            },
                                            Err(_) => {
                                                ("invalid post content: invalid UTF-8".as_bytes().to_vec(), 
                                                 "text/plain".into(), 500)
                                            }
                                        }
                                    }
                                },
                                // Handle dialog termination requests
                                HtmlDialogResourceType::TerminateRequest => {
                                    if let Some(ref b) = self.browser_monitor {
                                        b.borrow_mut().terminate();
                                    }
                                    info!("Terminated, exiting.");
                                    ("OK".as_bytes().to_vec(), "text/plain".into(), 200)
                                },
                            },
                            // Handle resource not found
                            None => (format!("Resource not found: {}", connection.get_location())
                                    .as_bytes().to_vec(), "text/plain".into(), 404)
                        };
                        connection.reset();
                        res
                    };
                    // Generate and send HTTP response
                    let length = content.len();
                    let response = match response {
                        200 => "HTTP/1.1 200 OK".into(),
                        404 => "HTTP/1.1 404 Not Found".into(),
                        500 => "HTTP/1.1 500 Internal Server Error".into(),
                        505 => "HTTP/1.1 505 HTTP Version Not Supported".into(),
                        0 => {
                            // Connection closed, don't reply
                            info!("Connection closed");
                            return false;
                        }
                        _ => format!("HTTP/1.1 {} Error", response)
                    };
                    // Create full HTTP response with headers
                    let response = format!(
                        "{}\r\nContent-Length: {}\r\nConnection: {}\r\nContent-Type: {}\r\n\r\n", 
                        response, length, 
                        if keep { "keep-alive" } else { "close" }, 
                        content_type
                    );
                    info!("==> {}", response);
                    // Write response and content
                    connection.write(response.as_bytes());
                    connection.write(&content);
                };
                if !keep {
                    info!("Closing connection");
                }
                // Return whether to keep the connection
                keep
            });            
            // Process browser messages if needed
            if let Some(ref browser_monitor) = self.browser_monitor {
                if browser_monitor.borrow_mut().pump_messages() {
                    any_action = true;
                }
            }            
            // Sleep if no activity to avoid busy-waiting
            if !any_action {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }        
        // Clean up browser monitor before returning
        self.browser_monitor = None;
        Ok(())
    }


    /// Get the handler for direct access.
    ///
    /// # Returns the handler for this dialog
    pub fn get_handler(&mut self) -> &mut T {
        &mut self.handler
    }
}
//! Edge WebView2-based browser implementation for the HTML Dialog system.
//!
//! This module provides a browser implementation using the Microsoft Edge WebView2 control,
//! which embeds the Chromium-based Edge browser in a native window. It's only available
//! on Windows with the WebView2 Runtime installed.
//!
//! The implementation provides:
//! - Window creation and management for the WebView2 control
//! - Event handling for navigation, document title, and favicon changes
//! - Browser lifetime management including COM initialization/cleanup
//! - Temporary cache folder management
//! - Parent window modal dialog functionality

use crate::browser::BrowserMonitor;
use std::{cell::RefCell, mem, path::{Path, PathBuf}, ptr, rc::Rc, sync::mpsc};
use super::message_loop;
use windows::{
    core::*,
    Win32::{
        Foundation::{E_POINTER, HINSTANCE, HWND, LPARAM, WPARAM, LRESULT, RECT, SIZE},
        Graphics::{Gdi, GdiPlus},
        System::{Com::*, LibraryLoader, Registry},
        UI::{
            Input::{KeyboardAndMouse},
            WindowsAndMessaging::{self, WINDOW_LONG_PTR_INDEX, WNDCLASSW, HICON},
        },
    },
};
use super::window::Window;
use webview2_com::{Microsoft::Web::WebView2::Win32::*, *};

/// Browser implementation using Microsoft Edge WebView2.
///
/// This struct encapsulates the WebView2 browser, its window, and the COM environment
/// required to run it. It manages the lifetime of all resources and implements the
/// `BrowserMonitor` trait to provide a consistent interface for the HTML dialog system.
struct BrowserEdgeView2 {
    /// Flag indicating if the browser has been terminated
    terminated: bool,
    /// Handle to the browser window
    window_handle: HWND,
    /// Flag indicating if COM was initialized by this instance and must be uninitialized on drop
    must_terminate_com: bool,
    /// The WebView2 control and its associated resources
    webview: Option<WebView>,
}

impl BrowserEdgeView2 {
    /// Creates a new WebView2-based browser.
    ///
    /// # Parameters
    /// * `parent` - Optional parent window to which this browser will be attached
    /// * `as_modal` - If true and a parent is provided, the parent window will be disabled
    ///                until this browser is closed, simulating a modal dialog
    /// * `url` - The URL to load in the browser
    ///
    /// # Returns
    /// * `Ok(Self)` - A new browser instance
    /// * `Err(...)` - If creation failed, with an error message
    ///
    /// # Errors
    /// Common error cases include:
    /// * COM initialization failures
    /// * WebView2 creation failures
    /// * Window creation failures
    pub fn new(parent: Option<super::window::Window>, as_modal: bool, url: String) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        // Initialize COM for this thread if needed
        let hr: HRESULT = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        let must_terminate_com = match hr {
            HRESULT(0) => {
                // S_OK we initialized COM, so we must uninitialize it later
                true        
            },
            HRESULT(1) => {
                // S_FALSE, COM was already initialized on this thread
                false
            },
            HRESULT(-2147417850) => {
                // RPC_E_CHANGED_MODE, COM was already initialized with a different mode, try to proceed
                false
            },
            hr => {
                return Err(format!("failed to initialize COM: {hr:?}").into());
            }
        };
        // Set debug flag based on build configuration
        #[cfg(debug_assertions)]
        let debug = true;
        #[cfg(not(debug_assertions))]
        let debug = false;
        // Disable the parent window if this is a modal dialog
        let mut must_enable_window = false;
        if let Some(ref p) = parent {
            if as_modal && p.handle != HWND::default() {
                unsafe {
                    let _ = KeyboardAndMouse::EnableWindow(p.handle, false);
                    must_enable_window = true;
                }
            };
        }
        // Create the WebView control
        let webview = match WebView::create(&url, parent, must_enable_window, debug) {
            Ok(wv) => wv,
            Err(e) => {
                // Clean up COM if we initialized it
                if must_terminate_com {
                    unsafe {
                        CoUninitialize();
                    }
                }
                return Err(e);
            }
        };
        
        // Return the new browser instance
        Ok(BrowserEdgeView2 {
            terminated: false,
            window_handle: *webview.frame,
            must_terminate_com,
            webview: Some(webview),
        })
    }
}

impl Drop for BrowserEdgeView2 {
    /// Clean up resources when the browser is dropped.
    ///
    /// This includes:
    /// * Cleaning up the WebView2 control
    /// * Removing temporary cache folders
    /// * Uninitializing COM if we initialized it
    fn drop(&mut self) {
        let cache_folder = self.webview.take().unwrap().clean_up();
        WebView::remove_cache_folder(&cache_folder);
        if self.must_terminate_com {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

impl BrowserMonitor for BrowserEdgeView2 {
    /// Checks if the browser has been terminated.
    ///
    /// # Returns
    /// * `Ok(true)` - If the browser has been terminated
    /// * `Ok(false)` - If the browser is still running
    fn is_terminated(&mut self) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        if let Some(webview) = &self.webview {
            if *webview.terminated.borrow() {
                self.terminated = true;
            }
        } else {
            self.terminated = true;
        }
        Ok(self.terminated)
    }
    
    /// Resizes the browser window to the specified dimensions.
    ///
    /// # Parameters
    /// * `width` - The desired width in pixels
    /// * `height` - The desired height in pixels
    ///
    /// # Returns
    /// * `Ok(())` - If the resize was successful
    /// * `Err(...)` - If the resize failed (e.g., no window handle)
    fn resize_request(&mut self, width: u32, height: u32) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if self.window_handle == HWND::default() {
            return Err("no window handle".into());
        }
        let _ = unsafe { WindowsAndMessaging::SetWindowPos(self.window_handle, None,
            0, 0, width as i32, height as i32,
            WindowsAndMessaging::SWP_NOACTIVATE | WindowsAndMessaging::SWP_NOZORDER | WindowsAndMessaging::SWP_NOMOVE,
        ) };
        Ok(())
    }
    
    /// Signals the browser to terminate.
    ///
    /// This posts a WM_CLOSE message to the browser window, which will trigger
    /// the window's close procedure.
    fn terminate(&mut self) {
        if self.window_handle != HWND::default() {
            let _ = unsafe { WindowsAndMessaging::PostMessageW(Some(self.window_handle), WindowsAndMessaging::WM_CLOSE, WPARAM(0), LPARAM(0)) };
        }
    }
    
    /// Gets the window handle associated with the browser.
    ///
    /// # Returns
    /// * `Some(Window)` - If the browser has a valid window handle
    /// * `None` - If no valid window handle is available
    fn get_window(&mut self) -> Option<Window> {
        if self.window_handle.0 == ptr::null_mut() {
            return None;
        }
        Some(Window{handle: self.window_handle})
    }

    /// Processes any pending window messages.
    ///
    /// # Returns
    /// * `true` - If any messages were processed
    /// * `false` - If no messages were processed
    fn pump_messages(&mut self) -> bool {
        message_loop::pump()
    }
}

/// Creates and starts a WebView2-based browser.
///
/// # Parameters
/// * `parent` - Optional parent window to which the browser will be attached
/// * `as_modal` - If true and parent is provided, the parent window will be disabled
/// * `url` - The URL to load in the browser
///
/// # Returns
/// * `Ok(Box<dyn BrowserMonitor>)` - A boxed browser monitor implementation
/// * `Err(...)` - If browser creation failed
pub(crate) fn start_browser_as_edgeview2(parent: Option<Window>, as_modal: bool, url: &str) -> std::result::Result<Box<dyn super::browser::BrowserMonitor>, Box<dyn std::error::Error>> {
    let browser = BrowserEdgeView2::new(parent, as_modal, url.to_string())?;
    return Ok(Box::new(browser));
}

/// Checks if the WebView2 runtime is installed.
///
/// This function checks the Windows registry to determine if the WebView2 runtime
/// is installed on the system. It follows Microsoft's recommended detection approach.
///
/// # Returns
/// * `true` - If the WebView2 runtime is installed
/// * `false` - If the WebView2 runtime is not installed
pub(crate) fn is_webview_installed() -> bool {
    // Check registry for WebView2 runtime installation
    // Registry path differs between 32-bit and 64-bit systems
    if cfg!(target_pointer_width = "64") {
        let mut key: Registry::HKEY = Registry::HKEY::default();
        let subkey = CoTaskMemPWSTR::from("Software\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}");
        if unsafe { Registry::RegOpenKeyW(Registry::HKEY_LOCAL_MACHINE, Some(subkey.as_ref().as_pcwstr()), &mut key).is_ok() } {
            unsafe { key.free() };
            return true;
        }
    }
    if cfg!(target_pointer_width = "32") {
        let mut key: Registry::HKEY = Registry::HKEY::default();
        let subkey = CoTaskMemPWSTR::from("Software\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}");
        if unsafe { Registry::RegOpenKeyW(Registry::HKEY_LOCAL_MACHINE, Some(subkey.as_ref().as_pcwstr()), &mut key).is_ok() } {
            unsafe { key.free() };
            return true;
        }
    }
    
    // Check user-specific installation as a fallback
    let mut key: Registry::HKEY = Registry::HKEY::default();
    let subkey = CoTaskMemPWSTR::from("Software\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}");
    if unsafe { Registry::RegOpenKeyW(Registry::HKEY_CURRENT_USER, Some(subkey.as_ref().as_pcwstr()), &mut key).is_ok() } {
        unsafe { key.free() };
        return true;
    }
    false
}

/// WebView2 wrapper that encapsulates the browser control and its resources.
///
/// This struct uses reference counting (Rc) for many of its fields because it is
/// cloned when setting up event handlers.
#[derive(Clone)]
pub struct WebView {
    /// Controller for the WebView2 instance
    controller: Rc<ICoreWebView2Controller>,
    /// The WebView2 core interface
    webview: Rc<ICoreWebView2>,
    /// Handle to the window that hosts the WebView2 control
    frame: Rc<HWND>,
    /// Window icon, shared between clones
    icon: Rc<RefCell<HICON>>,
    /// Path to temporary cache folder for this WebView2 instance
    cache_folder: Rc<PathBuf>,
    /// Flag indicating if the browser has been terminated
    terminated: Rc<RefCell<bool>>,
    /// Token for the navigation completed event handler
    navigation_complete_token: Rc<RefCell<i64>>,
    /// Reference to the parent window, if any
    parent: Rc<Option<super::window::Window>>,
    /// Flag indicating if the parent window needs to be re-enabled when this browser closes
    must_enable_window: bool,
}

impl WebView {
    /// Creates a new WebView instance.
    ///
    /// # Parameters
    /// * `url` - The URL to load in the browser
    /// * `parent` - Optional parent window
    /// * `must_enable_window` - Flag indicating if the parent window must be re-enabled on close
    /// * `debug` - Flag enabling debug features (context menus, dev tools) if true
    ///
    /// # Returns
    /// * `Ok(WebView)` - A new WebView instance
    /// * `Err(...)` - If creation failed
    pub fn create(url: &String, parent: Option<super::window::Window>, must_enable_window: bool, debug: bool) -> std::result::Result<WebView, Box<dyn std::error::Error>> {
        // Create a unique temporary directory for the WebView2 cache
        let cache_folder: PathBuf = loop {
            let temp_folder = Path::new(&std::env::temp_dir()).join(&uuid::Uuid::new_v4().to_string());
            if temp_folder.exists() {
                continue;
            }
            match std::fs::create_dir(&temp_folder) {
                Ok(_) => {
                   break temp_folder;
                },
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::AlreadyExists {
                        return Err(e.into());
                    }
                }
            }
        };
        
        // Create the WebView instance
        let webview = match Self::create_internal(url, parent, must_enable_window, cache_folder.clone(), debug) {
            Ok(wv) => wv,
            Err(e) => {
                Self::remove_cache_folder(&cache_folder);
                return Err(e);
            }
        };
        Ok(webview)
    }
    
    /// Removes a WebView2 cache folder.
    ///
    /// This function tries multiple times to delete the folder, as WebView2
    /// might keep a lock on it for a short time after being closed.
    ///
    /// # Parameters
    /// * `p` - Path to the cache folder to remove
    fn remove_cache_folder(p: &PathBuf) {
        // Try multiple times to delete the folder
        let mut attempt = 0;
        while p.exists() {
            match std::fs::remove_dir_all(p) {
                Ok(_) => {
                    break;
                },
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        break;
                    }
                    attempt = attempt + 1;
                    if attempt == 10 {
                        assert!(false, "failed to delete cache folder {p:?}: {e}");
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
    
    /// Internal implementation of WebView creation.
    ///
    /// This function handles the creation of the host window and its embedded WebView2 component.
    ///
    /// # Parameters
    /// * `url` - The URL to load in the browser
    /// * `parent` - Optional parent window
    /// * `must_enable_window` - Flag indicating if the parent window must be re-enabled on close
    /// * `cache_folder` - Path to use for the WebView2 cache
    /// * `debug` - Flag enabling debug features if true
    ///
    /// # Returns
    /// * `Ok(WebView)` - A new WebView instance
    /// * `Err(...)` - If creation failed at any step
    pub fn create_internal(url: &String, parent: Option<super::window::Window>, must_enable_window: bool, cache_folder: PathBuf, debug: bool) -> std::result::Result<WebView, Box<dyn std::error::Error>> {
        // Create a host window for the WebView2 control
        let frame = {
            let window_class = WNDCLASSW {
                lpfnWndProc: Some(Self::dialog_window_proc),
                lpszClassName: w!("BrowserDialogParent"),
                ..Default::default()
            };
            let parent_wnd = match parent {
                Some(ref p) => Some(p.handle),
                None => None,
            };
            unsafe {
                WindowsAndMessaging::RegisterClassW(&window_class);
                WindowsAndMessaging::CreateWindowExW(
                    Default::default(),
                    w!("BrowserDialogParent"),
                    w!("BrowserDialogParent"),
                    WindowsAndMessaging::WS_OVERLAPPEDWINDOW,
                    WindowsAndMessaging::CW_USEDEFAULT,
                    WindowsAndMessaging::CW_USEDEFAULT,
                    WindowsAndMessaging::CW_USEDEFAULT,
                    WindowsAndMessaging::CW_USEDEFAULT,
                    parent_wnd,
                    None,
                    LibraryLoader::GetModuleHandleW(None).ok().map(|h| HINSTANCE(h.0)),
                    None,
                )
            }.unwrap()
        };
        
        // Create the WebView2 environment
        let environment = {
            let (tx, rx) = mpsc::channel();
            let cache_folder_arg = cache_folder.clone();
            CreateCoreWebView2EnvironmentCompletedHandler::wait_for_async_operation(
                Box::new(move |environmentcreatedhandler| unsafe {
                    let browser_executable_folder: PWSTR = PWSTR::default(); // unused, use default installation
                    let enviroment_options = CoreWebView2EnvironmentOptions::default();
                    let enviroment_options: ICoreWebView2EnvironmentOptions = enviroment_options.into();
                    CreateCoreWebView2EnvironmentWithOptions(
                        browser_executable_folder,
                        &HSTRING::from(cache_folder_arg.as_os_str()),
                        &enviroment_options, 
                        &environmentcreatedhandler).map_err(webview2_com::Error::WindowsError)
                }),
                Box::new(move |error_code, environment| {
                    error_code?;
                    tx.send(environment.ok_or_else(|| windows::core::Error::from(E_POINTER))).expect("send over mpsc channel");
                    Ok(())
                }),
            )?;
            rx.recv().map_err(|e| e)?
        }?;
        // Create the WebView2 controller
        let controller = {
            let (tx, rx) = mpsc::channel();
            CreateCoreWebView2ControllerCompletedHandler::wait_for_async_operation(
                Box::new(move |handler| unsafe {
                    environment
                        .CreateCoreWebView2Controller(frame, &handler)
                        .map_err(webview2_com::Error::WindowsError)
                }),
                Box::new(move |error_code, controller| {
                    error_code?;
                    tx.send(controller.ok_or_else(|| windows::core::Error::from(E_POINTER)))
                        .expect("send over mpsc channel");
                    Ok(())
                }),
            )?;
            rx.recv().map_err(|e| e)?
        }?;
        // Configure the WebView2 controller
        let size = Self::get_window_size(frame);
        let mut client_rect = RECT::default();
        unsafe {
            let _ = WindowsAndMessaging::GetClientRect(frame, &mut client_rect);
            controller.SetBounds(RECT {
                left: 0,
                top: 0,
                right: size.cx,
                bottom: size.cy,
            })?;    
            controller.SetIsVisible(true)?;
        }
        // Get the core WebView2 interface
        let webview = unsafe { controller.CoreWebView2()? };
        // Apply advanced settings if available (WebView2 version-dependent)
        match controller.cast::<ICoreWebView2Controller4>() {
            Err(_) => {},
            Ok(controller4) => {
                let _ = unsafe { controller4.SetAllowExternalDrop(false) };
                let _ = unsafe { controller4.SetDefaultBackgroundColor(COREWEBVIEW2_COLOR {R: 255, G: 255, B: 255, A: 0}) };
            }
        }
        // Configure debug settings
        if !debug {
            unsafe {
                let settings = webview.Settings()?;
                settings.SetAreDefaultContextMenusEnabled(false)?;
                settings.SetAreDevToolsEnabled(false)?;
            }
        }
        // Create the WebView wrapper
        let webview = WebView {
            controller: Rc::new(controller),
            webview: Rc::new(webview),
            frame: Rc::new(frame),
            icon: Rc::new(RefCell::new(HICON::default())),
            cache_folder: Rc::new(cache_folder),
            terminated: Rc::new(RefCell::new(false)),
            navigation_complete_token: Rc::new(RefCell::new(0)),
            parent: Rc::new(parent),
            must_enable_window
        };
        // Set up the favicon changed event handler (if available)
        match webview.webview.cast::<ICoreWebView2_15>() {
            Err(_) => {},
            Ok(webview15) => {
                let mut _token = 0;
                let wv = webview.clone();
                unsafe { webview15.add_FaviconChanged(
                    &FaviconChangedEventHandler::create(Box::new(move |webview, _args| {
                        // Get the favicon
                        let wv = wv.clone();
                        if let Some(webview) = webview {
                            match webview.cast::<ICoreWebView2_15>() {
                                Err(_) => {},
                                Ok(webview15) => {
                                    webview15.GetFavicon(COREWEBVIEW2_FAVICON_IMAGE_FORMAT_PNG,
                                       &GetFaviconCompletedHandler::create(Box::new(move |_webview, _args| {
                                           if let Some(icon_stream) = _args {
                                                let mut gdip_token: usize = 0;
                                                match GdiPlus::GdiplusStartup(&mut gdip_token as *mut usize, &GdiPlus::GdiplusStartupInput {
                                                        GdiplusVersion: 1,
                                                        DebugEventCallback: 0,
                                                        SuppressBackgroundThread: false.into(),
                                                        SuppressExternalCodecs: false.into(),
                                                    }, ptr::null_mut()) {
                                                    GdiPlus::Status(0) => {
                                                        let mut bitmap: *mut GdiPlus::GpBitmap = std::ptr::null_mut();
                                                        let stat = GdiPlus::GdipCreateBitmapFromStream(&icon_stream, &mut bitmap as *mut *mut GdiPlus::GpBitmap);
                                                        match stat {
                                                           GdiPlus::Status(0) => {
                                                                let mut icon: HICON = HICON::default();
                                                                match GdiPlus::GdipCreateHICONFromBitmap(bitmap, &mut icon as *mut HICON) {
                                                                    GdiPlus::Status(0) => {
                                                                        wv.set_icon(icon);
                                                                    },
                                                                    _ => {}
                                                                };
                                                               GdiPlus::GdipDisposeImage(bitmap as *mut GdiPlus::GpImage);
                                                           },
                                                           _ => {}
                                                       };
                                                       // Shut down GDI+
                                                       GdiPlus::GdiplusShutdown(gdip_token);
                                                    },
                                                    _ => {}
                                               };
                                           };
                                           Ok(())
                                       }))
                                    ).unwrap_or_default();
                                }
                            };
                        }
                        Ok(())
                    })),
                    &mut _token)}.unwrap_or_default();
            },
        }
        // Set up the document title changed event handler
        let mut _token = 0;
        let wv = webview.clone();
        unsafe { webview.webview.add_DocumentTitleChanged(
            &DocumentTitleChangedEventHandler::create(Box::new(move |webview, _args| {
                // Get the document title
                if let Some(webview) = webview {
                    let mut title: PWSTR = PWSTR::default();
                    match webview.DocumentTitle(&mut title as *mut PWSTR) {
                        Ok(_) => {
                            wv.set_title(title);
                            CoTaskMemFree(Some(title.0 as *const std::ffi::c_void));
                        },
                        _ => {},
                    }
                }
                Ok(())
            })),
            &mut _token)
        }.unwrap_or_default();
        // Associate the WebView with its window
        WebView::set_window_webview(frame, Some(Box::new(webview.clone())));
        // Set up the navigation completed event handler
        let mut wv = webview.clone();
        let handler = NavigationCompletedEventHandler::create(Box::new(move |_sender, _args| {
            wv.navigation_complete();
            Ok(())
        }));
        let mut token = 0;
        unsafe { webview.webview.add_NavigationCompleted(&handler, &mut token) }?;
        *webview.navigation_complete_token.borrow_mut() = token;
        // Navigate to the initial URL
        let url = CoTaskMemPWSTR::from(url.as_str());
        unsafe { webview.webview.Navigate(*url.as_ref().as_pcwstr()) }?;
        Ok(webview)
    }

    /// Handles navigation completion.
    ///
    /// This method is called when page navigation completes. It removes the navigation
    /// event handler (since it's only needed once) and shows the browser window.
    fn navigation_complete(&mut self) {
        let mut token = self.navigation_complete_token.borrow_mut();
        if *token != 0 {
            let _ = unsafe { self.webview.remove_NavigationCompleted(*token) };
            *token = 0;
            // Show window
            let frame = *self.frame;
            unsafe {
                let _ = WindowsAndMessaging::ShowWindow(frame, WindowsAndMessaging::SW_SHOW);
                let _ = Gdi::UpdateWindow(frame);
                let _ = KeyboardAndMouse::SetFocus(Some(frame));
            }
        }
    }

    /// Window procedure for the WebView2 host window.
    ///
    /// This function handles window messages for the WebView2 host window.
    ///
    /// # Parameters
    /// * `hwnd` - Window handle
    /// * `msg` - Message identifier
    /// * `w_param` - Message-specific parameter
    /// * `l_param` - Message-specific parameter
    ///
    /// # Returns
    /// * `LRESULT` - Message-specific result
    extern "system" fn dialog_window_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        let webview = match WebView::get_window_webview(hwnd) {
            Some(webview) => webview,
            None => return unsafe { WindowsAndMessaging::DefWindowProcW(hwnd, msg, w_param, l_param) },
        };
        match msg {
            WindowsAndMessaging::WM_SIZE => {
                let size = WebView::get_window_size(hwnd);
                unsafe {
                    webview.controller.SetBounds(RECT {
                            left: 0,
                            top: 0,
                            right: size.cx,
                            bottom: size.cy,
                        })
                        .unwrap();
                }
                LRESULT::default()
            }
            WindowsAndMessaging::WM_CLOSE => {
                // Re-enable the parent window if this is a modal dialog
                if webview.must_enable_window {
                    if let Some(ref p) = *webview.parent {
                        if p.handle != HWND::default() {
                            let _ = unsafe { KeyboardAndMouse::EnableWindow(p.handle, true) };
                        }
                    }
                }
                unsafe {
                    let _ = WindowsAndMessaging::DestroyWindow(hwnd);
                }
                LRESULT::default()
            }
            WindowsAndMessaging::WM_DESTROY => {
                // Mark as terminated and clean up
                *webview.terminated.borrow_mut() = true;
                // Deallocate the WebView structure that is in raw via SetWindowLong
                WebView::set_window_webview(hwnd, None); 
                LRESULT::default()
            }
            _ => unsafe { WindowsAndMessaging::DefWindowProcW(hwnd, msg, w_param, l_param) },
        }
    }

    /// Sets the icon for the WebView2 window.
    ///
    /// # Parameters
    /// * `icon` - The icon handle to set
    fn set_icon(&self, icon: HICON) {
        let mut my_icon = self.icon.borrow_mut();
        if *my_icon != HICON::default() {
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::DestroyIcon(*my_icon);
            }
        }
        *my_icon = icon;
        unsafe {
            let _ = WindowsAndMessaging::SendMessageW(
                *self.frame,
                WindowsAndMessaging::WM_SETICON,
                Some(WPARAM(WindowsAndMessaging::ICON_SMALL as usize)),
                Some(LPARAM(icon.0 as isize)),
            );
            let _ = WindowsAndMessaging::SendMessageW(
                *self.frame,
                WindowsAndMessaging::WM_SETICON,
                Some(WPARAM(WindowsAndMessaging::ICON_BIG as usize)),
                Some(LPARAM(icon.0 as isize)),
            );
        }
    }

    /// Sets the title of the WebView2 window.
    ///
    /// # Parameters
    /// * `title` - The title to set
    pub fn set_title(&self, title: PWSTR) {
        unsafe {
            let _ = WindowsAndMessaging::SetWindowTextW(*self.frame, title);
        }
    }

    /// Associates a WebView instance with a window.
    ///
    /// Stores the WebView instance as user data in the window, allowing it to be
    /// retrieved in the window procedure.
    ///
    /// # Parameters
    /// * `hwnd` - The window handle
    /// * `webview` - The WebView to associate, or None to remove the association
    ///
    /// # Returns
    /// * The previous WebView instance associated with the window, if any
    fn set_window_webview(hwnd: HWND, webview: Option<Box<WebView>>) -> Option<Box<WebView>> {
        unsafe {
            match Self::SetWindowLong(
                hwnd,
                WindowsAndMessaging::GWLP_USERDATA,
                match webview {
                    Some(webview) => Box::into_raw(webview) as _,
                    None => 0_isize,
                },
            ) {
                0 => None,
                ptr => Some(Box::from_raw(ptr as *mut _)),
            }
        }
    }

    /// Retrieves the WebView instance associated with a window.
    ///
    /// # Parameters
    /// * `hwnd` - The window handle
    ///
    /// # Returns
    /// * `Some(Box<WebView>)` - The associated WebView instance
    /// * `None` - If no WebView is associated with the window
    fn get_window_webview(hwnd: HWND) -> Option<Box<WebView>> {
        unsafe {
            let data = Self::GetWindowLong(hwnd, WindowsAndMessaging::GWLP_USERDATA);
            match data {
                0 => None,
                _ => {
                    let webview_ptr = data as *mut WebView;
                    let raw = Box::from_raw(webview_ptr);
                    let webview = raw.clone();
                    mem::forget(raw);
                    Some(webview)
                }
            }
        }
    }
    /// Sets window user data for 32-bit platforms.
    #[allow(non_snake_case)]
    #[cfg(target_pointer_width = "32")]
    unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
        WindowsAndMessaging::SetWindowLongW(window, index, value as _) as _
    }
    /// Sets window user data for 64-bit platforms.
    #[allow(non_snake_case)]
    #[cfg(target_pointer_width = "64")]
    unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
        unsafe { WindowsAndMessaging::SetWindowLongPtrW(window, index, value) }
    }
    /// Gets window user data for 32-bit platforms.
    #[allow(non_snake_case)]
    #[cfg(target_pointer_width = "32")]
    unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
        WindowsAndMessaging::GetWindowLongW(window, index) as _
    }
    /// Gets window user data for 64-bit platforms.
    #[allow(non_snake_case)]
    #[cfg(target_pointer_width = "64")]
    unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
        unsafe { WindowsAndMessaging::GetWindowLongPtrW(window, index) }
    }

    /// Gets the client area size of a window.
    ///
    /// # Parameters
    /// * `hwnd` - The window handle
    ///
    /// # Returns
    /// * `SIZE` - The window's client area size
    fn get_window_size(hwnd: HWND) -> SIZE {
        let mut client_rect = RECT::default();
        let _ = unsafe { WindowsAndMessaging::GetClientRect(hwnd, &mut client_rect) };
        SIZE {
            cx: client_rect.right - client_rect.left,
            cy: client_rect.bottom - client_rect.top,
        }
    }

    /// Cleans up resources used by the WebView.
    ///
    /// This method is called when the WebView is no longer needed. It cannot
    /// be implemented in Drop because the struct is cloned multiple times.
    ///
    /// # Returns
    /// * `PathBuf` - The path to the cache folder that should be deleted
    pub fn clean_up(self) -> PathBuf {
        // Clean up icon resources
        let mut my_icon = self.icon.borrow_mut();
        if *my_icon != HICON::default() {
            unsafe {
                let _ = windows::Win32::UI::WindowsAndMessaging::DestroyIcon(*my_icon);
            }
        }
        *my_icon = HICON::default(); // Value is shared - ensure it's not cleaned up by another instance
        // Terminate the WebView
        let _ = unsafe { self.controller.Close() };
        // Return the cache folder for deletion
        (*self.cache_folder).clone()
    }
}
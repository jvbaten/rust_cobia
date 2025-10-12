#![windows_subsystem = "windows"]

//for windows and debug, we log to debug console
#[cfg(target_os = "windows")]
#[cfg(debug_assertions)]
use win_dbg_logger;
#[cfg(target_os = "windows")]
#[cfg(debug_assertions)]
use log;

mod user_info;
mod user_photo;

fn main() {
    //server log to debug console (windows, debug mode only)
    #[cfg(target_os = "windows")]
    #[cfg(debug_assertions)]
    {
        //write http server log to debug console
        win_dbg_logger::init();
        //write INFO level, then we we all interaction with the http server
        log::set_max_level(log::LevelFilter::Debug);
    }

    //show the main dialog
    let mut user_info_dlg=user_info::UserInfoDialogHandler::new();
    match user_info_dlg.show() {
        Ok(_) => {},
        Err(e) => {eprintln!("Error: {}",e)}
    }
}
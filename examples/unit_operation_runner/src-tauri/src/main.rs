// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
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
    //parse command line
    let mut filename = None;
    let mut do_run = false;
    let mut msg= None;
    let mut first = true;
    for arg in std::env::args() {
        if first {
            first=false;
            #[cfg(target_os = "windows")]
            continue;
            #[cfg(not(target_os = "windows"))]
            {
                //check if this is indeed the executable name
                if arg.ends_with("unitoperationrunner") || arg.ends_with("unitoperationrunner.exe") {
                    continue;
                }
            }
        }
        if arg == "--run" {
            if do_run {
                msg=Some("--run specified multiple times".to_string());
            }
            do_run = true;
        } else if arg.starts_with("-") {
            msg=Some(format!("Unknown argument {}", arg));
        } else {
            if filename.is_some() {
                msg=Some("Multiple file names specified".to_string());
            }
            filename = Some(arg);
        }
        first=false;
    }
    if msg.is_none() && do_run && filename.is_none() {
        msg=Some("--run specified but no file name provided".to_string());
    }
    if let Some(ref filename) = filename {
        if !std::path::Path::new(&filename).exists() {
            msg=Some(format!("File '{}' does not exist", filename));
        }
    }
    if let Some(msg) = msg {
        let msg=format!("{}\n\nUsage: unitoperationrunner [--run] [filename]\n\n--run: Run the specified file without showing the UI\nfilename: The file to run (optional, required for --run)", msg);    
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
    }
    //if --run is specified, run the file and exit (may panic)
    if do_run {
        unitoperationrunner_lib::run_file(filename.unwrap());
        std::process::exit(0);
    }
    unitoperationrunner_lib::run(filename);
    unreachable!();
}

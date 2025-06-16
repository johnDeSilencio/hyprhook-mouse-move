use fork::{Fork, daemon};
use std::{ffi::OsStr, process::Command};

use serde::Deserialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const TRIGGER_FILE_PATH: &str = "/home/nicholas/.waybar_is_running";
const WAYBAR_INACTIVE_MOUSE_Y_POSITION_THRESHOLD: f64 = 10.0;
const WAYBAR_ACTIVE_MOUSE_Y_POSITION_THRESHOLD: f64 = 100.0;

#[derive(Debug, Deserialize)]
struct MousePosition {
    #[serde(alias = "x")]
    _x: f64,
    y: f64,
}

fn main() {
    let args = std::env::args();
    let mouse_position = get_mouse_position(args);
    let waybar_is_running = trigger_file_exists();

    if waybar_is_running {
        if mouse_position.y >= WAYBAR_ACTIVE_MOUSE_Y_POSITION_THRESHOLD {
            // Specifically construct here to improve performance
            let mut sys = System::new();
            sys.refresh_processes_specifics(
                ProcessesToUpdate::All,
                false,
                ProcessRefreshKind::nothing(),
            );

            let waybar_process = get_waybar_process(&sys);
            let killed_process = match waybar_process {
                Some(waybar_process) => waybar_process.kill(),
                _ => true,
            };

            if !killed_process {
                panic!("failed to kill waybar process");
            }

            // Remove the trigger file so we know that Waybar is not running on the next invocation
            std::fs::remove_file(TRIGGER_FILE_PATH).expect("failed to remove trigger file");
        }
    } else {
        if mouse_position.y <= WAYBAR_INACTIVE_MOUSE_Y_POSITION_THRESHOLD {
            // Waybar wasn't running before and the mouse has now moved to the top of the screen.
            // Start Waybar
            if let Ok(Fork::Child) = daemon(false, false) {
                Command::new("waybar")
                    .output()
                    .expect("waybar command failed to start");
            }

            // Create the trigger file so we know Waybar is running on the next invocation
            std::fs::File::create(TRIGGER_FILE_PATH).expect("failed to create trigger file");
        }
    }
}

fn get_mouse_position(args: std::env::Args) -> MousePosition {
    let unparsed_mouse_position = match args.into_iter().nth(1) {
        Some(mouse_position) => mouse_position,
        None => {
            panic!(
                "ERROR: Expected exactly one command-line argument with mouse position in JSON format"
            );
        }
    };

    let mouse_position: MousePosition = match serde_json::from_str(unparsed_mouse_position.as_str())
    {
        Ok(mouse_position) => mouse_position,
        _ => {
            panic!(
                "ERROR: Expected command-line argument in format '{{\"x\": 123.456, \"y\": 789.012}}'",
            );
        }
    };

    mouse_position
}

fn trigger_file_exists() -> bool {
    match std::fs::exists(TRIGGER_FILE_PATH) {
        Ok(exists) => exists,
        _ => panic!("failed to check if the trigger file exists or not"),
    }
}

fn get_waybar_process<'a>(sys: &'a sysinfo::System) -> Option<&'a sysinfo::Process> {
    match sys.processes_by_name(OsStr::new("waybar")).nth(0) {
        Some(process) => Some(process),
        None => None,
    }
}

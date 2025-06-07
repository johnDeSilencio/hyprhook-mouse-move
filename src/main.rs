use fork::{Fork, daemon};
use std::process::Command;

use serde::Deserialize;
use sysinfo::System;

#[derive(Debug, Deserialize)]
struct MousePosition {
    #[serde(alias = "x")]
    _x: f64,
    y: f64,
}

fn main() {
    let args = std::env::args();
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

    let mut sys = System::new_all();

    // First we update all information of our `System` struct.
    sys.refresh_all();

    let mut waybar_is_running = false;

    for (_, process) in sys.processes() {
        match process.name().to_str() {
            Some(process_name) => {
                if process_name.contains("waybar") {
                    waybar_is_running = true;

                    if mouse_position.y >= 20.0 {
                        // Waybar is running and the mouse is no longer at the top
                        // of the screen. Kill Waybar
                        process.kill();
                    }

                    break;
                }
            }
            // If we can't convert process name to &str, just skip it
            _ => continue,
        };
    }

    if !waybar_is_running && mouse_position.y < 20.0 {
        // Waybar is not running but the mouse is at the top of the screen.
        // Start Waybar
        if let Ok(Fork::Child) = daemon(false, false) {
            Command::new("waybar")
                .output()
                .expect("waybar command failed to start");
        }
    }
}

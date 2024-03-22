use std::process::Command;

use log::{error, info};



pub fn kill_exe(exe_name: &str) {
    if cfg!(windows) {
        match Command::new("taskkill")
        .args(&["/IM", exe_name, "/F"])
        .spawn() {
            Ok(_) => {
                info!("{} was closed",exe_name)
            },
            Err(e) => {
                error!("Failed to execute taskkill {}: {}",exe_name, e)
            },
        }

      }
}
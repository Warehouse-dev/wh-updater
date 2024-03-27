use anyhow::{anyhow, Result};
use async_compat::Compat;
use crc32fast::Hasher;
use log::{error, info, warn};
use native_dialog::FileDialog;
use std::{
    f32::consts::E,
    fs::{self, File},
    io::{self, stdin, Read},
    path::{Path, PathBuf},
    process::exit,
};
use winreg::{enums::*, RegKey};

use crate::{WHGames, WHUpdateClient};

pub fn update_foc() {
    info!("Game: Transformers Fall of Cybertron");
    //info!("Press enter to continue...");
    //stdin().read(&mut [0]).unwrap();

    info!("Getting game instalation path...");

    let game_path = match get_foc_path() {
        Ok(game_path) => game_path,
        Err(_) => exit(1),
    };

    info!(
        "Game instalation path: '{}'",
        &game_path.to_str().unwrap_or_default()
    ); //Better way to do it?

    info!("Checking if any updates are necessary...");
    let mut updater = WHUpdateClient::new(game_path.clone(), WHGames::FOC);

    match updater.is_update_required() {
        Ok(is_update_required) => match is_update_required {
            true => {

                let client_update_path =match smol::block_on(Compat::new(async {
                    updater
                        .download_update_zip()
                        .await
                        
                })) {
                    Ok(clinet_update_path) => clinet_update_path,
                    Err(e) => {
                        error!("Failed to download update files! ({e})");
                        exit(1)
                    },
                };
                println!("{:?}",client_update_path);               
                let _ = updater.unpack_update(client_update_path);
                updater.create_backup();
                updater.apply_update();


            }
            false => {}
        },
        Err(_) => {
            error!("Failed to update. Aborting.");
            exit(1)
        }
    };

    //foc_coal_update(game_path, updater);
}

fn get_foc_path() -> Result<PathBuf> {
    //For TFOC we need to aquire the root dir, not Binaries.

    match fs::metadata(Path::new("./TFOC.exe")) {
        Ok(_) => {
            info!("Found TFOC.exe!");
            let mut path = PathBuf::from(".");
            path.pop();
            return Ok(path);
        }
        Err(_) => {}
    }

    match fs::metadata(Path::new("./Binaries/TFOC.exe")) {
        Ok(_) => {
            info!("Found TFOC.exe!");
            return Ok(PathBuf::from("."));
        }
        Err(_) => {}
    }

    match get_foc_instalation_path_from_registry() {
        Some(mut registry_path) => {
            registry_path.push("Binaries");
            registry_path.push("TFOC.exe");

            match fs::metadata(&registry_path) {
                Ok(_) => {
                    info!("Found TFOC.exe!");
                    registry_path.pop();
                    registry_path.pop();
                    return Ok(registry_path);
                }
                Err(_) => {}
            }
        }
        None => {}
    }

    warn!("Failed to find TFOC installation path automatically. Please select it manually.");

    match foc_folder_picker_dialog() {
        Some(path) => return Ok(path),
        None => {}
    }

    Err(anyhow!("Failed to find TFOC installation folder. Exiting."))
}

pub fn get_foc_instalation_path_from_registry() -> Option<PathBuf> {
    let foc_reg = {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(reg_key) =
            hklm.open_subkey("SOFTWARE\\WOW6432Node\\Activision\\Transformers Fall of Cybertron")
        {
            reg_key
        } else if let Ok(reg_key) =
            hklm.open_subkey("Software\\Activision\\Transformers Fall of Cybertron")
        {
            reg_key
        } else {
            log::error!("Transformers Fall of Cybertron path is missing from registry.");
            return None;
        }
    };

    // should be for steam C:\Program Files (x86)\Steam\steamapps\common\Transformers Fall of Cybertron
    // and whatever for others

    let Ok(installpath) = foc_reg.get_value::<String, _>("installpath") else {
        log::error!("Transformers Fall of Cybertron doesn't have installpath key.");
        return None;
    };

    return Some(PathBuf::from(installpath));
}

fn foc_folder_picker_dialog() -> Option<PathBuf> {
    if let Some(path_temp) = FileDialog::new().show_open_single_dir().unwrap() {
        //Covering case where Binaries folder is selected

        let mut tmp = path_temp.clone();

        tmp.push("TFOC.exe");

        match fs::metadata(&tmp) {
            Ok(_) => {
                info!("Found TFOC.exe!");
                tmp.pop();
                tmp.pop();
                return Some(tmp);
            }
            Err(_) => {}
        }

        //covering case where root folder is selected

        let mut tmp = path_temp.clone();

        tmp.push("Binaries");
        tmp.push("TFOC.exe");

        match fs::metadata(&tmp) {
            Ok(_) => {
                info!("Found TFOC.exe!");
                tmp.pop();
                tmp.pop();
                return Some(tmp);
            }
            Err(_) => {}
        }
        None
    } else {
        None
    }
}

fn foc_coal_update(game_path: PathBuf, mut wh_updater: WHUpdateClient) {
    //check local coal if it's one of default coals an replace them

    info!("Checking if Coalesced.ini need an update...");

    let mut file_path = game_path.clone();

    file_path.push("TransGame");
    file_path.push("Config");
    file_path.push("PC");
    file_path.push("Cooked");
    file_path.push("Coalesced.ini");

    let exists = &file_path.try_exists().unwrap_or(false);

    if !exists {
        error!(
            "Coalesced.ini doesn't exists at the expected path of {}.",
            &file_path.to_str().unwrap_or_default()
        );
        return;
    };

    let mut file = match File::open(&file_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to open Coalesced.ini - {e}");
            return;
        }
    };

    let mut file_buf: Vec<u8> = vec![];

    match file.read_to_end(&mut file_buf) {
        Ok(bytes) => {}
        Err(e) => {
            error!("Failed to read Coalesced.ini - {e}");
            return;
        }
    };

    let checksum = crc32fast::hash(&file_buf); //This is far from ideal, but will do

    let default_steam_coal_hash: u32 = 0x0db5f77e;
    let wh_coal_hash: u32 = 0x09571ded; //for 21.03.2024
    let _default_nonsteam_coal_hash: u32 = 1; //is this even a thing?

    //TODO:
    // break apart this function into get_hash and updater partsh
    // move to ratatui (or just straight up to tauri)

    if checksum == default_steam_coal_hash {
        info!("Coalesced.ini is a default one. Performing an update.");

        let coal_url = "https://wiki.aiwarehouse.xyz/guides/tfcfoc_guide_upd/coalesced.ini"; //Move it to s3?

        let mut coal_path = game_path.clone();
        coal_path.push("coalesced.ini");

        match smol::block_on(Compat::new(wh_updater.download_file(coal_url, &coal_path))) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to download Coalesced.ini - {}", e);
                return;
            }
        };

        //backing up orig coal
        let _ = fs::rename(
            &file_path,
            &game_path
                .join("TransGame")
                .join("Config")
                .join("PC")
                .join("Cooked")
                .join("Coalesced.ini.orig.bak"),
        );

        //moving downloaded coal

        let _ = fs::rename(&coal_path, file_path);
    }
    if checksum == wh_coal_hash {
        info!("Latest WH coal detected, skipping.");
        return;
    } else {
        warn!("Custom coal detected. No changes will be done.");
        return;
    }
}

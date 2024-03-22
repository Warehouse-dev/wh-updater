use anyhow::{anyhow, Result};
use log::{info, warn};
use std::{fs, path::PathBuf};
use std::time::Duration;
use tempfile::tempdir;
use version_compare::{compare_to, Cmp};

mod cdn_driver;
mod fs_tools;
mod utils;

pub mod foc;



pub struct WHUpdateClient {
    http_client: reqwest::blocking::Client,
    pub temp_path: PathBuf,
    pub game_path: PathBuf,
    pub game: WHGames,
    update_files: Vec<String>,
    pub remote_version: String,
    pub local_version: Option<String>,
    pub update_base_url: String
}

pub enum WHGames {
    FOC,
    WFC,
    ROTDS,
    ROTF,
    GRID,
    GH3
}

impl WHUpdateClient {
    pub fn new(game_path: PathBuf, game: WHGames) -> Self {
        let http_client = reqwest::blocking::Client::builder()
            .https_only(true)
            .use_rustls_tls()
            .connect_timeout(Duration::from_secs(5))
            .gzip(true)
            .user_agent("Warehouse Updater Client v0.1")
            .build()
            .unwrap();

        let temp_path = tempdir().unwrap().into_path();

        fs::remove_dir_all(&temp_path).unwrap_or_default();
        fs::create_dir_all(&temp_path).expect("Failed to create temp dir");

        let update_base_url = match game {
            WHGames::FOC => "https://aiwarehouse.fra1.cdn.digitaloceanspaces.com/wh/foc",
            WHGames::WFC => todo!(),
            WHGames::ROTDS => todo!(),
            WHGames::ROTF => todo!(),
            WHGames::GRID => todo!(),
            WHGames::GH3 => todo!(),
        }.to_owned();

        let update_files = vec![];
        let remote_version = String::from("0.0.0.0");
        
        Self {
            http_client,
            temp_path,
            game_path,
            game,
            update_files,
            remote_version,
            local_version: None,
            update_base_url
        }
    }

    pub fn is_update_required(&mut self) -> Result<bool> {
        let version_remote = match self.get_remote_version() {
            Ok(version_remote) => version_remote,
            Err(e) => {
                return Err(anyhow!(
                    "Failed to get remote version - {}. Aborting update",
                    e.to_string()
                ))
            }
        };

        let version_local = match self.get_local_version() {
            Ok(version_local) => version_local,
            Err(e) => {
                warn!("Failed to get version of local files.");
                String::from("None")
            }
        };
        info!(
            "Remote version - {} | Local version - {}",
            version_remote, version_local
        );

        if version_local == "None" {
            return Ok(true);
        }

        let update_needed = compare_to(version_remote, version_local, Cmp::Gt).unwrap_or(false);

        if update_needed {
            return Ok(true);
        } else {
            return Ok(false);
        }

        Ok(false)
    }

    

    fn get_local_version(&mut self) -> Result<String> {
        let version_path = &self.game_path.join("version");

        match fs::read_to_string(version_path) {
            Ok(local_version) => Ok(local_version),
            Err(e) => return Err(anyhow!(e.to_string())),
        }
    }



}

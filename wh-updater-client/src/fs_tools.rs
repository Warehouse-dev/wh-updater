use std::{fs, io, path::{Path, PathBuf}};
use anyhow::{anyhow, Result};
use log::info;
use crate::WHUpdateClient;




impl WHUpdateClient {
    pub fn unpack_update(&mut self, path: PathBuf) -> Result<Vec<PathBuf>> {
        let fname = path;

        let base_path = match fname.parent() {
            Some(path) => path.to_owned(),
            None => PathBuf::new(),
        };

        let mut extracted_files: Vec<PathBuf> = vec![];

        let file = fs::File::open(fname).unwrap();

        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let temp_path = base_path.join(path.to_owned());

                    match path.parent() {
                        Some(parent) => {
                            if parent == Path::new("") {
                                self.update_files.push(path.to_str().unwrap().to_owned())
                            }
                        }
                        None => {}
                    };

                    extracted_files.push(temp_path.to_owned());
                    temp_path
                }
                None => continue,
            };

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    println!("File {i} comment: {comment}");
                }
            }

            if (*file.name()).ends_with('/') {
                println!("File {} extracted to \"{}\"", i, outpath.display());
                fs::create_dir_all(&outpath).unwrap();
            } else {
                println!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );

                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }

        Ok(extracted_files)
    }


    pub fn list_zip_files(update_zip_path: PathBuf) {

        let file = fs::File::open(update_zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            println!("{:?}",file.enclosed_name());
    }
}   

    pub fn unpack_zip(path: PathBuf) -> Result<Vec<PathBuf>> {
        let fname = path;

        let base_path = match fname.parent() {
            Some(path) => path.to_owned(),
            None => PathBuf::new(),
        };

        let mut extracted_files: Vec<PathBuf> = vec![];

        let file = fs::File::open(fname).unwrap();

        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = match file.enclosed_name() {
                Some(path) => {
                    let temp_path = base_path.join(path.to_owned());

                    extracted_files.push(temp_path.to_owned());
                    temp_path
                }
                None => continue,
            };

            {
                let comment = file.comment();
                if !comment.is_empty() {
                    println!("File {i} comment: {comment}");
                }
            }

            if (*file.name()).ends_with('/') {
                println!("File {} extracted to \"{}\"", i, outpath.display());
                fs::create_dir_all(&outpath).unwrap();
            } else {
                println!(
                    "File {} extracted to \"{}\" ({} bytes)",
                    i,
                    outpath.display(),
                    file.size()
                );

                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p).unwrap();
                    }
                }
                let mut outfile = fs::File::create(&outpath).unwrap();
                io::copy(&mut file, &mut outfile).unwrap();
            }

            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }

        Ok(extracted_files)
    }

    pub fn apply_update(&mut self) {
        info!("Applying an update...");
        for file_path in &self.update_files {

            let _ = fs::rename(
                &self.temp_path.join(file_path),
                &self.game_path.join(file_path),
            );
        }

        let mut version_file =
            fs::File::create(&self.game_path.join("version")).unwrap();
        io::copy(&mut self.remote_version.as_bytes(), &mut version_file).unwrap();
        info!("Done!");
    }

    pub fn create_backup(&mut self) {
            info!("Performing a backup...");
            // list_to_backup should be generated previously

            let _ = fs::remove_dir_all(&self.game_path.join(".wh_bak"));
            let _ = fs::create_dir_all(&self.game_path.join(".wh_bak"));

            for file_to_backup in self.update_files.clone() {

                let temp_path = &self.game_path.join(&file_to_backup);

                let _ = fs::rename(
                    temp_path,
                    &self.game_path.join(".wh_bak").join(&file_to_backup),
                );
            }


            let _ = fs::rename(
                &self.game_path.join("version"),
                &self
                    .game_path
                    .join(".wh_bak")
                    .join("version"),
            );        
            info!("Done!");
    }

}
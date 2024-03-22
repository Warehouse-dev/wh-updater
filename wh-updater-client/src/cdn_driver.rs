use std::path::{Path, PathBuf};
use std::{cmp::min, fmt::Write, fs::File, time::Duration};
use std::io::Write as io_write;
use crate::WHUpdateClient;
use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use log::{debug, warn};

impl WHUpdateClient {

    pub fn get_remote_version(&mut self) -> Result<String> {
    
        let url = self.url_gen("/version.ver");

        let response = match self
            .http_client
            .get(url)
            .send()
        {
            Ok(response) => response,
            Err(e) => return Err(anyhow!(e.to_string())),
        };

        match response.text() {
            Ok(version_string) => {
                let temp = version_string.replace("\n", "");
                self.remote_version = temp.to_owned();
                Ok(temp)
            }
            Err(e) => return Err(anyhow!(e.to_string())),
        }
    }

    pub async fn download_update_zip(&mut self) -> Result<PathBuf> {

        let url = self.url_gen("/client_files.zip");

        let update_zip_path = self.temp_path.join("client_files.zip");

        match self.download_file(&url, &update_zip_path).await {
            Ok(_) => Ok(update_zip_path),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }



    pub async fn download_file(&mut self, url: &str, path: &Path) -> Result<(), String> {

        let client = reqwest::Client::builder()
            .https_only(true)
            .use_rustls_tls()
            .connect_timeout(Duration::from_secs(5))
            .gzip(true)
            .user_agent("WH Updater Client v0.1")
            .build()
            .unwrap();

        // Reqwest setup
        let res = client
            .get(url)
            .send()
            .await
            .or(Err(format!("Failed to GET from '{}'", &url)))?;

        match res
            .content_length(){
                Some(total_size) => {                    

                    let pb = ProgressBar::new(total_size);
                    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                    .progress_chars("#>-"));
            
                    // download chunks
                    let mut file = File::create(path).or(Err(format!("Failed to create file '{}'", path.to_str().unwrap_or_default())))?;
                    let mut downloaded: u64 = 0;
                    let mut stream = res.bytes_stream();
            
                    while let Some(item) = stream.next().await {
                        let chunk = item.or(Err(format!("Error while downloading file")))?;
                        file.write_all(&chunk)
                            .or(Err(format!("Error while writing to file")))?;
                        let new = min(downloaded + (chunk.len() as u64), total_size);
                        downloaded = new;
                        pb.set_position(new);
                    }
            
                    pb.finish_with_message(format!("Downloaded {}", url));
                    return Ok(());
                },
                None => {
                    debug!("Failed to get content length from '{}'", &url);

                    let file_data = res.bytes().await.or(Err(format!("Error while downloading file")))?;

                    let mut file = File::create(path).or(Err(format!("Failed to create file '{}'", path.to_str().unwrap_or_default())))?;
                    
                    let _ = file.write(&file_data);

                    return Ok(());
                },
            };
       
    }


    fn url_gen(&mut self, url: &str) -> String {
        format!("{}{url}",self.update_base_url)
    }

}
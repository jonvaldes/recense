use chrono::prelude::*;
use failure::Error;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pin {
    pub id: String,
    pub title: String,
    pub urls: Vec<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub created: DateTime<Utc>,
}

impl Pin {
    pub fn new() -> Pin {
        let now = Utc::now();
        Pin {
            id: sha1::Sha1::from(format!("{}", now.timestamp_nanos())).hexdigest(),
            title: String::from(""),
            urls: vec![],
            description: String::new(),
            tags: vec![],
            created: now,
        }
    }

    pub fn fill_defaults(&mut self) {
        let default_pin = Pin::new();

        if self.title.is_empty() {
            if !self.urls.is_empty() {
                self.title = self.urls[0].clone();
            } else {
                self.title = default_pin.title;
            }
        }
    }
}

struct DownloadRequest {
    url: String,
    pin_id: String,
    username: String,
}

#[derive(Clone)]
pub struct BackingStore {
    in_channel: mpsc::Sender<DownloadRequest>,
}

impl BackingStore {
    fn downloader_thread(channel: mpsc::Receiver<DownloadRequest>) {
        loop {
            let download_request = channel.recv().unwrap();

            println!("Getting url: {}", download_request.url);

            /*
            let output =std::process::Command::new("chromium-browser")
                .arg("--headless")
                .arg("--disable-gpu")
                .arg("--window-size=1280,1696")
                .arg("--screenshot")
                .arg("--dump-dom")
                .arg(&download_request.url)
                .output()
                .expect("Failed to run chromium");

            // TODO - MOVE screenshot.png to the right place

            let screenshot_filename = BackingStore::pin_filename("png", &download_request.username, &download_request.pin_id);


            let filename = BackingStore::pin_filename("html", &download_request.username, &download_request.pin_id);
            if let Err(x) = std::fs::write(filename, &output.stdout) {
                println!("Error writing w3m output: {}", x);
            }
            */
        }
    }

    pub fn new() -> BackingStore {
        let (in_channel, out_channel) = mpsc::channel();
        std::thread::spawn(move || BackingStore::downloader_thread(out_channel));

        BackingStore { in_channel }
    }

    pub fn add_pin(&self, username: String, pin: Pin) -> Result<(), Error> {
        let mut pin = pin;
        pin.fill_defaults();

        let pin_json = serde_json::to_string(&pin).unwrap();
        let filename = BackingStore::pin_filename("json", &username, &pin.id);

        std::fs::create_dir_all(BackingStore::pin_directory(&username))?;

        println!("Filename: {}", filename);
        std::fs::write(filename, &pin_json)?;

        if pin.urls.len() > 0 {
            self.in_channel
                .send(DownloadRequest {
                    url: pin.urls[0].clone(),
                    pin_id: pin.id,
                    username,
                })
                .unwrap();
        }

        Ok(())
    }

    fn pin_filename(extension: &str, username: &str, id: &str) -> String {
        format!("pins/{}/{}_v0.{}", username, id, extension)
    }

    fn pin_directory(username: &str) -> String {
        format!("pins/{}/", username)
    }

    pub fn get_pin(&self, username: String, id: String) -> Result<Pin, Error> {
        let filename = BackingStore::pin_filename("json", &username, &id);
        self.get_pin_from_filename(&filename)
    }

    pub fn get_pin_from_filename(&self, filename: &str) -> Result<Pin, Error> {
        let json_data = std::fs::read_to_string(&std::path::Path::new(&filename))?;
        Ok(serde_json::from_str(&json_data)?)
    }

    pub fn get_all_tags(&self, username: &str) -> Result<Vec<(String, usize)>, Error> {
        let pins = self.get_all_pins(username)?;

        let mut result = std::collections::HashMap::<String,usize>::new();
        pins.iter().map(|p| p.tags.clone()).flatten().for_each(|tag|{
            let counter = result.entry(tag).or_insert(0);
            *counter += 1;
        });

        let mut result_vec : Vec<(String, usize)>= result.iter().map(|(k,v)| (k.clone(),v.clone())).collect();
        result_vec.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(result_vec)
    }

    pub fn get_all_pins(&self, username: &str) -> Result<Vec<Pin>, Error> {
        let path_str = BackingStore::pin_directory(username);
        let dir_path = std::path::Path::new(&path_str);

        if !dir_path.exists() {
            return Ok(vec![]);
        }

        let pins_res: Result<Vec<Pin>, Error> = std::fs::read_dir(dir_path)?
            .filter(|file| {
                if !file.is_ok() {
                    return false;
                }
                let file = file.as_ref().unwrap();
                let path = file.path();
                let extension = path.as_path().extension().clone();
                if extension.is_none() {
                    return false;
                }
                extension.unwrap() == "json"
            })
            .map(|file| {
                let file = file.unwrap();
                self.get_pin_from_filename(&file.path().as_path().to_str().unwrap())
            })
            .collect();

        if pins_res.is_err() {
            return pins_res;
        }

        let mut pins = pins_res.unwrap();
        pins.sort_by(|a, b| b.created.cmp(&a.created));

        Ok(pins)
    }

    pub fn search_pins(&self, username: &str, search_pattern: &str) -> Result<Vec<Pin>, Error> {
        let pins = self.get_all_pins(username)?;

        let search_terms = search_pattern.split_whitespace();

        Ok(pins
            .iter()
            .filter(|p| {
                p.title.contains(search_pattern)
                    || p.urls
                        .iter()
                        .any(|u| search_terms.clone().all(|term| u.contains(term)))
                    || search_terms
                        .clone()
                        .all(|term| p.description.contains(term))
                    || p.tags
                        .iter()
                        .any(|tag| search_terms.clone().all(|term| tag.contains(term)))
            })
            .cloned()
            .collect())
    }
}

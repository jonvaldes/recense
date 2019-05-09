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
    pub starred: bool,
    pub unread: bool,
    pub created: DateTime<Utc>,
}

impl Pin {
    pub fn new() -> Pin {
        Pin {
            id: sha1::Sha1::from(Utc::now().to_rfc3339()).hexdigest(),
            title: String::from(""),
            urls: vec![],
            description: String::new(),
            tags: vec![],
            starred: false,
            unread: true,
            created: Utc::now(),
        }
    }

    pub fn id_from_url(url: &str) -> String {
        let mut sha = sha1::Sha1::new();
        sha.update(url.as_bytes());
        sha.hexdigest()
    }

    pub fn fill_defaults(&mut self) {
            
        let default_pin = Pin::new();

        if self.title.is_empty() {
            if !self.urls.is_empty() {
                self.title = self.urls[0].clone();
            }else{
                self.title = default_pin.title;
            }
        }
        if !self.urls.is_empty() {
            let mut sha = sha1::Sha1::new();
            self.urls.iter().for_each(|url| sha.update(url.as_bytes()));
            self.id = sha.hexdigest();
        }else{
            self.id = default_pin.id;
        }
    }
}

#[derive(Clone)]
pub struct BackingStore {
    in_channel: mpsc::Sender<String>,
}

impl BackingStore {
    fn downloader_thread(channel: mpsc::Receiver<String>) {
        loop {
            let new_url = channel.recv().unwrap();

            println!("Getting url: {}", new_url);

            let output = std::process::Command::new("w3m")
                .arg(&new_url)
                .arg("-dump")
                .output()
                .expect("Failed to run w3m");
       
            let id = Pin::id_from_url(&new_url);
            let filename = BackingStore::pin_filename("txt", "jon", &id);
            if let Err(x) = std::fs::write(filename, &output.stdout){
                println!("Error writing w3m output: {}", x);
            }
        }
    }

    pub fn new() -> BackingStore {
        let (in_channel, out_channel) = mpsc::channel();
        std::thread::spawn(move || BackingStore::downloader_thread(out_channel));

        BackingStore { in_channel }
    }

    pub fn add_pin(&self, pin: Pin) -> Result<(), Error>{

        let mut pin = pin;
        pin.fill_defaults();

        let pin_json = serde_json::to_string(&pin).unwrap();
        let filename = BackingStore::pin_filename("json", "jon", &pin.id);
        println!("Filename: {}", filename);
        std::fs::write(filename, &pin_json)?;
       
        if pin.urls.len() > 0 {
            self.in_channel.send(pin.urls[0].clone()).unwrap();
        }

        Ok(())
    }

    fn pin_filename(extension:&str, username: &str, id: &str) -> String {
        format!("pins/{}/{}_v0.{}", username, id, extension)
    }

    pub fn get_pin(&self, id: String) -> Result<Pin, Error> {
        let username = "jon";
        let filename = BackingStore::pin_filename("json", username, &id);
        self.get_pin_from_filename(&filename)
    }

    pub fn get_pin_from_filename(&self, filename: &str) -> Result<Pin, Error> {
        let json_data = std::fs::read_to_string(&std::path::Path::new(&filename))?;
        Ok(serde_json::from_str(&json_data)?)
    }

    pub fn get_all_pins(&self, username: &str) -> Result<Vec<Pin>, Error> {
        std::fs::read_dir(format!("pins/{}", username))?
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
            .collect()
    }
}


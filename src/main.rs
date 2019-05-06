/*
 * PLAN: Bookmarking service
 * - Initially backed by a directory
 * - Each bookmark is a JSON (?) file
 * - External API:
 *      - Add bookmark: you just submit a URL
 *          - Internally downloads link, renders it with w3m, stores output
 *      - Remove bookmark
 *      - Get bookmarks: Returns all titles, URLs and peek data
 *      - Search: Searches through bookmark contents
 *
 * - Backing:
 *      - Directory with bookmarks. Each with a UUID name.
 *      <UUID>_<version>.json // Metadata
 *      <UUID>_<version>.txt // Contents
 *
 *
 * - Taking screenshots: Supposedly Firefox can take screenshots in a "headless" mode it has. I've
 * been unable to make it work, though. In theory this should be how you do that:
 *      firefox -no-remote -url https://valdes.cc/ -screenshot test.jpg
 *      (add "-P <profilename>" to make it use another profile and allow several instances of
 *      Firefox running)
 *      See: https://developer.mozilla.org/en-US/docs/Mozilla/Firefox/Headless_mode
 *
 *
 * TODO 
 * -----
 * - Implement getting all pins
 * - Implement searching through pins
 * - Implement getting a website's title
 *
 * - Handle adding the same URL twice
 * - Save output of w3m alongside json data
 * - 
 * - Create per-user directories
 * - Add authentication for several users
 * - 
 */
extern crate actix_web;
extern crate chrono;
extern crate serde;
extern crate serde_json;
extern crate sha1;

#[macro_use]
extern crate failure;

use actix_web::{http, server, App, State, HttpRequest, Responder, Query};
use chrono::prelude::*;
use failure::Error;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;

use std::io::{self, Write};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Pin {
    id: String,
    name: String,
    urls: Vec<String>,
    description: String,
    tags: Vec<String>,
    starred: bool,
    unread: bool,
    created: DateTime<Utc>,
}

impl Pin {
    fn new() -> Pin {
        Pin {
            id: sha1::Sha1::from(Utc::now().to_rfc3339()).hexdigest(),
            name: String::from(""),
            urls: vec![],
            description: String::new(),
            tags: vec![],
            starred: false,
            unread: true,
            created: Utc::now(),
        }
    }

    fn id_from_url(url: &str) -> String {
        let mut sha = sha1::Sha1::new();
        sha.update(url.as_bytes());
        sha.hexdigest()
    }

    fn fill_defaults(&mut self) {
            
        let default_pin = Pin::new();

        if self.name.is_empty() {
            if !self.urls.is_empty() {
                self.name = self.urls[0].clone();
            }else{
                self.name = default_pin.name;
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
struct BackingStore {
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

#[derive(Clone)]
struct AppState {
    storage: BackingStore,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            storage: BackingStore::new(),
        }
    }
}

fn index(req: HttpRequest<AppState>) -> impl Responder {
    "OK"
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PinInfo {
    name: Option<String>,
    url: Option<String>,
    description: Option<String>,
    tags: Option<String>, // %20-separated
    starred: Option<bool>,
    unread: Option<bool>,
}


fn add_pin(state: State<AppState>, pin_info: Query<PinInfo>) -> impl Responder {

    let pin_info = pin_info.into_inner();
    let mut pin = Pin::new();

    if let Some(name) = pin_info.name {
        pin.name = name;
    }
    if let Some(url) = pin_info.url {
        pin.urls = vec!(url);
    }
    if let Some(description) = pin_info.description {
        pin.description = description;
    }
    if let Some(tags) = pin_info.tags {
        pin.tags = tags.split_whitespace().filter(|x| !x.is_empty()).map(|x| String::from(x)).collect();
    }
    if let Some(starred) = pin_info.starred {
        pin.starred = starred;
    }
    if let Some(unread) = pin_info.unread {
        pin.unread = unread;
    }

    if let Err(err) = state.storage.add_pin(pin) {
        println!("Err: {:?}", err);
    }

    actix_web::dev::HttpResponseBuilder::new(actix_web::http::StatusCode::OK).finish()
}

fn main() {
    server::new(|| {
        let initial_state = AppState::new();

        App::<AppState>::with_state(initial_state)
            .route("/", http::Method::GET, index)
            .route("/add_pin", http::Method::GET, add_pin)
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run();
}

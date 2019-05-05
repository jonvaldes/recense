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
 * - Create per-user directories
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
    fn new_from_url(url: &str) -> Pin {
        Pin {
            id: sha1::Sha1::from(url).hexdigest(),
            name: String::from(url),
            urls: vec![String::from(url)],
            description: String::new(),
            tags: vec![],
            starred: false,
            unread: true,
            created: Utc::now(),
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

            //TODO -- check output.status
            println!("Output: {}", String::from_utf8(output.stdout).expect("TODO -- accept non-utf8 crap"));
        }
    }

    pub fn new() -> BackingStore {
        let (in_channel, out_channel) = mpsc::channel();
        std::thread::spawn(move || BackingStore::downloader_thread(out_channel));

        BackingStore { in_channel }
    }

    pub fn add_pin_form_url(&self, url: String) -> Result<(), Error>{

        let pin = Pin::new_from_url(&url);
        let pin_json = serde_json::to_string(&pin).unwrap();
        let filename = BackingStore::pin_filename("jon", &pin.id);
        println!("Filename: {}", filename);
        std::fs::write(filename, &pin_json)?;
        
        self.in_channel.send(url).unwrap();

        Ok(())
    }

    fn pin_filename(username: &str, id: &str) -> String {
        format!("pins/{}/{}_v0.json", username, id)
    }

    pub fn get_pin(&self, id: String) -> Result<Pin, Error> {
        let username = "jon";
        let filename = BackingStore::pin_filename(username, &id);
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
    req.state()
        .storage
        .add_pin_form_url(String::from("https://www.google.com"));
    "OK"
}

#[derive(Deserialize)]
struct AddUrlInfo{
    url: String,
}

fn add_url(state: State<AppState>, info: Query<AddUrlInfo>) -> impl Responder {
    println!("Called");
    if let Err(err) = state.storage.add_pin_form_url(info.url.clone()) {
        println!("Err: {:?}", err);
    }

    actix_web::dev::HttpResponseBuilder::new(actix_web::http::StatusCode::OK).finish()
}

fn main() {
    server::new(|| {
        let initial_state = AppState::new();

        App::<AppState>::with_state(initial_state)
            .route("/", http::Method::GET, index)
            .route("/add_url", http::Method::GET, add_url)
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run();
}

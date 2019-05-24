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
    pub rendered_description: Option<String>,
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
            rendered_description: Some(String::new()),
            tags: vec![],
            created: now,
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


fn take_screenshot(browser_cmd: &str, req: &DownloadRequest) -> Result<(), Error> {

    // Dump screenshot
    std::fs::create_dir_all(format!("cache/{}", req.username))?;

    let window_width = 1280;
    let aspect_ratio = 1.0/2.0;
    let thumb_ratio = 5;
    let scrollbar_width = 20;

    let window_height = (window_width as f32 * aspect_ratio) as u32;

    std::fs::remove_file("screenshot.png").unwrap_or(());
    std::process::Command::new(browser_cmd)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg(format!("--window-size={},{}", window_width, window_height))
        .arg("--screenshot")
        .arg(&req.url)
        .output()
        .map_err(|e|{
            error!("Could not execute chromium to extract screenshot {}", e);
            e
        })?;

    // Move screenshot.png to the right place
    let screenshot_filename = format!("cache/{}/{}.jpg", &req.username, &req.pin_id);
    let html_filename = format!("cache/{}/{}.html", &req.username, &req.pin_id);

    {
        let mut screenshot = image::open("screenshot.png")?;
        {
            let cropped_screenshot = image::imageops::crop(&mut screenshot, 0,0, window_width - scrollbar_width, window_height);
            let thumbnail = image::imageops::thumbnail(&cropped_screenshot, (window_width - scrollbar_width)/thumb_ratio, window_height/thumb_ratio);
            thumbnail
                .save(&screenshot_filename)
                .map_err(|e|{
                    error!("Could not save screenshot file to filename {}. Error: {}", screenshot_filename, e);
                    e
                })?;
        }
    }
    // Dump DOM contents
    let output = std::process::Command::new(browser_cmd)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--dump-dom")
        .arg(&req.url)
        .output().map_err(|e| {
            error!("Could not execute chromium to extract html {}", e);
            e
        })?;

    std::fs::write(html_filename, &output.stdout).map_err(|e|{
        error!("Could not write browser's stdout: {}", e);
        e
    })?;

    Ok(())
}

impl BackingStore {
    fn downloader_thread(channel: mpsc::Receiver<DownloadRequest>) {

        let browsers = vec!("chromium", "chromium-browser", "google-chrome");

        loop {
            let download_request = channel.recv().unwrap();

            println!("Getting url: {}", download_request.url);

            for browser in &browsers {
                match take_screenshot(browser, &download_request){
                    Ok(_) => break,
                    Err(x) => error!("Error trying to invoke browser: {}\n{}", x, x.backtrace()),
                }
            }
        }
    }

    pub fn new() -> BackingStore {
        let (in_channel, out_channel) = mpsc::channel();
        std::thread::spawn(move || BackingStore::downloader_thread(out_channel));

        BackingStore { in_channel }
    }

    pub fn add_pin(&self, username: String, pin: Pin) -> Result<(), Error> {
        let mut pin = pin;

        // Fix up url
        if pin.urls.len() > 0 && pin.urls[0].len() > 0 {
            if !(pin.urls[0].starts_with("http://") || pin.urls[0].starts_with("https://")) {
                pin.urls[0] = format!("http://{}", pin.urls[0]);
            }
        }

        // Generate rendered markdown description
        pin.rendered_description =
            match super::htmlrenderer::render_markdown_string(&pin.description) {
                Err(err) => {
                    error!("Error rendering markdown description: {}", err);
                    None
                },
                Ok(x) => Some(x),
            };

        let pin_json = serde_json::to_string(&pin).unwrap();
        let filename = BackingStore::pin_filename("json", &username, &pin.id);

        std::fs::create_dir_all(BackingStore::pin_directory(&username))?;

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

    pub fn delete_pin(&self, username: &str, id: &str) -> Result<(), Error> {
        let filename = BackingStore::pin_filename("json", &username, id);

        if !std::path::Path::new(&filename).exists() {
            bail!("Pin id {} does not exist", id);
        }

        std::fs::remove_file(filename)?;

        Ok(())
    }

    fn pin_filename(extension: &str, username: &str, id: &str) -> String {
        format!("pins/{}/{}_v0.{}", username, id, extension)
    }

    fn pin_directory(username: &str) -> String {
        format!("pins/{}/", username)
    }

    pub fn get_pin(&self, username: &str, id: &str) -> Result<Pin, Error> {
        let filename = BackingStore::pin_filename("json", username, id);
        self.get_pin_from_filename(&filename)
    }

    pub fn get_pin_from_filename(&self, filename: &str) -> Result<Pin, Error> {
        let json_data = std::fs::read_to_string(&std::path::Path::new(&filename))?;
        Ok(serde_json::from_str(&json_data)?)
    }

    pub fn get_all_tags(&self, username: &str) -> Result<Vec<(String, usize)>, Error> {
        let pins = self.get_all_pins(username)?;

        let mut result = std::collections::HashMap::<String, usize>::new();
        pins.iter()
            .map(|p| p.tags.clone())
            .flatten()
            .for_each(|tag| {
                let counter = result.entry(tag).or_insert(0);
                *counter += 1;
            });

        let mut result_vec: Vec<(String, usize)> =
            result.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
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

        let search_pattern = search_pattern.to_lowercase();
        let search_terms = search_pattern.split_whitespace();

        Ok(pins
            .iter()
            .filter(|p| {
                let title = p.title.to_lowercase();

                title.contains(&search_pattern)
                    || p.urls.iter().any(|u| {
                        let url = u.to_lowercase();
                        search_terms.clone().all(|term| url.contains(term))
                    })
                    || search_terms
                        .clone()
                        .all(|term| p.description.to_lowercase().contains(term))
                    || p.tags.iter().any(|tag| {
                        let tag = tag.to_lowercase();
                        search_terms.clone().all(|term| tag.contains(term))
                    })
            })
            .cloned()
            .collect())
    }
}

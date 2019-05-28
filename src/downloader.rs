use actix_web::http;
use failure::Error;
use std::sync::mpsc;

pub struct DownloadRequest {
    pub url: String,
    pub pin_id: String,
    pub username: String,
}

fn take_screenshot(browser_cmd: &str, req: &DownloadRequest) -> Result<(), Error> {
    // Dump screenshot
    std::fs::create_dir_all(format!("cache/{}", req.username))?;

    let window_width = 1280;
    let aspect_ratio = 1.0 / 2.0;
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
        .map_err(|e| {
            error!("Could not execute chromium to extract screenshot {}", e);
            e
        })?;

    // Move screenshot.png to the right place
    let screenshot_filename = format!("cache/{}/{}.jpg", &req.username, &req.pin_id);

    {
        let mut screenshot = image::open("screenshot.png")?;
        {
            let cropped_screenshot = image::imageops::crop(
                &mut screenshot,
                0,
                0,
                window_width - scrollbar_width,
                window_height,
            );
            let thumbnail = image::imageops::thumbnail(
                &cropped_screenshot,
                (window_width - scrollbar_width) / thumb_ratio,
                window_height / thumb_ratio,
            );
            thumbnail.save(&screenshot_filename).map_err(|e| {
                error!(
                    "Could not save screenshot file to filename {}. Error: {}",
                    screenshot_filename, e
                );
                e
            })?;
        }
    }

    Ok(())
}

fn fix_html_references(handle: &mut html5ever::rcdom::Handle, server_url: &str) {
    let node = handle;
    match node.data {
        html5ever::rcdom::NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            if name.local.eq_str_ignore_ascii_case("a")
                || name.local.eq_str_ignore_ascii_case("link")
            {
                for attr in attrs.borrow_mut().iter_mut() {
                    if attr.name.local.eq_str_ignore_ascii_case("href") {
                        if String::from(attr.value.clone()).starts_with("/") {
                            attr.value = html5ever::tendril::Tendril::format(format_args!(
                                "{}{}",
                                server_url, attr.value
                            ));
                        }
                        println!(" {}=\"{}\"", attr.name.local, attr.value);
                    }
                }
            }
        }
        _ => (),
    }

    for mut child in node.children.borrow_mut().iter_mut() {
        fix_html_references(&mut child, server_url);
    }
}

fn download_link_source(browser_cmd: &str, req: &DownloadRequest) -> Result<(), Error> {
    println!("Downloading link source");
    let html_filename = format!("cache/{}/{}.html", &req.username, &req.pin_id);

    let url = req.url.parse::<http::uri::Uri>()?;
    let url_authority = match url.authority_part() {
        None => bail!("Can't download empty url {}", req.url),
        Some(x) => x,
    };

    let scheme = match url.scheme_part() {
        Some(x) => x.as_str(),
        None => "http",
    };

    let server_url = format!("{}://{}", scheme, url_authority.as_str());

    // Dump DOM contents
    let output = std::process::Command::new(browser_cmd)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--dump-dom")
        .arg(&req.url)
        .output()
        .map_err(|e| {
            error!("Could not execute chromium to extract html {}", e);
            e
        })?;

    use html5ever::tendril::TendrilSink;

    let opts = html5ever::driver::ParseOpts {
        tree_builder: html5ever::tree_builder::TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut dom = html5ever::parse_document(html5ever::rcdom::RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut &output.stdout[..])
        .unwrap();

    // Iterate over all elements of the DOM, and change all hrefs, etc
    fix_html_references(&mut dom.document, &server_url);

    let mut out = Vec::<u8>::new();
    html5ever::serialize(&mut out, &dom.document, Default::default())?;

    std::fs::write(html_filename, &out).map_err(|e| {
        error!("Could not write browser's stdout: {}", e);
        e
    })?;

    Ok(())
}

pub fn downloader_thread(channel: mpsc::Receiver<DownloadRequest>) {
    let browsers = vec![
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/usr/bin/google-chrome",
    ];
    let mut active_browser = "";
    for browser in browsers {
        if std::path::Path::new(&browser).exists() {
            active_browser = browser;
            break;
        }
    }

    loop {
        let download_request = channel.recv().unwrap();

        println!("Getting url: {}", download_request.url);

        let sshot_result = take_screenshot(active_browser, &download_request);
        let source_result = download_link_source(active_browser, &download_request);

        if let Err(x) = sshot_result {
            error!(
                "Error trying to generate screenshot: {}\n{}",
                x,
                x.backtrace()
            );
        }

        if let Err(x) = source_result {
            error!("Error trying to download source: {}\n{}", x, x.backtrace());
        }
    }
}

extern crate actix_web;
extern crate argon2rs;
extern crate chrono;
extern crate env_logger;
extern crate pulldown_cmark;
extern crate rand_pcg;
extern crate serde;
extern crate sha1;

#[macro_use]
extern crate failure;
#[macro_use]
extern crate handlebars;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;

use actix_web::middleware::{identity::RequestIdentity, Logger};
use actix_web::{fs::NamedFile, http, server, App, Form, HttpRequest, Responder, State};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::rc::Rc;

mod htmlrenderer;
mod pins;
mod user;
use pins::*;

#[derive(Clone)]
struct AppState {
    storage: BackingStore,
    html_renderer: Rc<htmlrenderer::HTMLRenderer>,
}

impl AppState {
    fn new() -> AppState {
        AppState {
            storage: BackingStore::new(),
            html_renderer: Rc::new(htmlrenderer::HTMLRenderer::new()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct PinInfo {
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
    tags: Option<String>, // %20-separated
    starred: Option<String>,
    unread: Option<String>,
}

fn add_pin(
    req: HttpRequest<AppState>,
    state: State<AppState>,
    pin_info: Form<PinInfo>,
) -> impl Responder {
    println!("got to add_pin");

    if req.identity() == None {
        error!("add_pin reached without a proper identity");
        return actix_web::HttpResponse::Forbidden().finish();
    }

    let pin_info = pin_info.into_inner();
    println!("Pin info: {:?}", pin_info);
    let mut pin = Pin::new();

    if let Some(title) = pin_info.title {
        pin.title = title;
    }
    if let Some(url) = pin_info.url {
        pin.urls = vec![url];
    }
    if let Some(description) = pin_info.description {
        pin.description = description;
    }
    if let Some(tags) = pin_info.tags {
        pin.tags = tags
            .split_whitespace()
            .filter(|x| !x.is_empty())
            .map(|x| String::from(x))
            .collect();
    }
    if let Some(starred) = pin_info.starred {
        pin.starred = starred == "on";
    }
    if let Some(unread) = pin_info.unread {
        pin.unread = unread == "on";
    }

    if let Err(err) = state.storage.add_pin(req.identity().unwrap(), pin) {
        error!("Err: {:?}", err);
    }

    actix_web::HttpResponse::SeeOther()
        .header(actix_web::http::header::LOCATION, "/")
        .finish()
}

/*
fn get_all_pins(state: State<AppState>) -> impl Responder {

    let username = "jon";

    if let Err(err) = state.storage.get_all_pins(username) -> Result<Vec<Pin>, Error> {
        println!("Err: {:?}", err);
        actix_web::dev::HttpResponseBuilder::new(actix_web::http::StatusCode::OK).finish()
    }

}
*/

fn login_screen(state: &AppState) -> actix_web::HttpResponse {
    use std::borrow::Borrow;
    let renderer: &htmlrenderer::HTMLRenderer = state.html_renderer.borrow();

    let mut data = std::collections::HashMap::new();
    data.insert(String::from("dummy"), String::from("dummy"));

    let contents = match renderer.render_page("login", &data) {
        Err(x) => {
            error!("{}",x);
            return actix_web::HttpResponse::InternalServerError().finish();
        },
        Ok(x) => x,
    };

    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(contents)
}

fn index(req: HttpRequest<AppState>) -> actix_web::HttpResponse {
    let username = req.identity().unwrap_or(String::new());

    if username == "" {
        return login_screen(req.state());
    }

    use std::borrow::Borrow;
    let renderer: &htmlrenderer::HTMLRenderer = req.state().html_renderer.borrow();

    let pins = match req.state().storage.get_all_pins(&username) {
        Err(err) => {
            error!("Err: {:?}", err);
            return actix_web::dev::HttpResponseBuilder::new(actix_web::http::StatusCode::OK)
                .finish();
        }
        Ok(x) => x,
    };

    let index_data = json!({
        "username": username.clone(),
        "pins": pins,
    });

    let contents = match renderer.render_page("index", &index_data) {
        Err(err) => {
            error!("Err: {:?}", err);
            return actix_web::HttpResponse::InternalServerError().finish();
        }
        Ok(x) => x,
    };

    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(contents)
}

fn todo(req: HttpRequest<AppState>) -> actix_web::HttpResponse {

    use std::borrow::Borrow;
    let renderer: &htmlrenderer::HTMLRenderer = req.state().html_renderer.borrow();

    let items = match htmlrenderer::render_markdown_file("TODO.md") {
        Err(err) => {
            error!("Err: {:?}", err);
            return actix_web::HttpResponse::InternalServerError().finish();
        },
        Ok(x) => x,
    };

    let todo_data = json!({"items": items});

    let contents = match renderer.render_page("todo", &todo_data) {
        Err(err) => {
            error!("Err: {:?}", err);
            return actix_web::HttpResponse::InternalServerError().finish();
        },
        Ok(x) => x,
    };

    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(contents)
}


fn static_files(req: HttpRequest<AppState>) -> actix_web::Result<NamedFile> {
    let path: PathBuf = req.match_info().query("path")?;
    Ok(NamedFile::open(format!(
        "static/{}",
        path.as_path().to_str().unwrap()
    ))?)
}

#[derive(Deserialize)]
struct SignupInfo {
    username: String,
    password: String,
    email: String,
}

fn signup(form: Form<SignupInfo>) -> actix_web::HttpResponse {
    let signup_info = form.into_inner();

    if let Err(x) = user::UserInfo::new_user(
        signup_info.username,
        signup_info.email,
        signup_info.password,
    ) {
        error!("Error trying to create new user: {}", x);
        actix_web::HttpResponse::InternalServerError().finish()
    } else {
        actix_web::HttpResponse::Ok().finish()
    }
}

#[derive(Debug, Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

fn login(form: Form<LoginInfo>, req: HttpRequest<AppState>) -> actix_web::HttpResponse {
    let login_info = form.into_inner();

    let user = match user::UserInfo::load_user_data(&login_info.username) {
        Err(x) => {
            error!("Could not get user data: {:?}. Error: {:?}", login_info, x);
            return actix_web::HttpResponse::Unauthorized().finish();
        }
        Ok(x) => x,
    };

    if user.verify_password(login_info.password) {
        req.remember(login_info.username); // TODO -- Can we store this directly, or do we have to store a secure token?
        actix_web::HttpResponse::SeeOther()
            .header(actix_web::http::header::LOCATION, "/")
            .finish()
    } else {
        actix_web::HttpResponse::Unauthorized().finish()
    }
}

fn logout(req: HttpRequest<AppState>) -> actix_web::HttpResponse {
    req.forget(); // <- remove identity
    actix_web::HttpResponse::SeeOther()
        .header(actix_web::http::header::LOCATION, "/")
        .finish()
}

fn main() {
    //std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_LOG", "pinroar=debug,actix_web=debug,handlebars=debug");
    env_logger::init();

    let cookie_key = {
        use rand_pcg::rand_core::RngCore;
        let mut cookie_key = vec![0u8; 32];

        let timestamp = Utc::now().timestamp_nanos();
        rand_pcg::Mcg128Xsl64::new(0x1337f00dd15ea5e5u128 + timestamp as u128)
            .fill_bytes(&mut cookie_key);
        cookie_key
    };

    server::new(move || {
        let initial_state = AppState::new();

        App::<AppState>::with_state(initial_state)
            .middleware(Logger::default())
            .middleware(actix_web::middleware::identity::IdentityService::new(
                // <- create identity middleware
                actix_web::middleware::identity::CookieIdentityPolicy::new(&cookie_key)
                    .name("auth-cookie")
                    .secure(false),
            ))
            //            .route("/get_all_pins", http::Method::GET, get_all_pins)
            .route("/", http::Method::GET, index)
            .route("/todo", http::Method::GET, todo)
            .route("/static/{path:.*}", http::Method::GET, static_files)
            .route("/signup", http::Method::POST, signup)
            .route("/login", http::Method::POST, login)
            .route("/logout", http::Method::POST, logout)
            .route("/add_pin", http::Method::POST, add_pin)
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .run();
}

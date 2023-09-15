use std::{
    collections::HashMap,
    fs::File,
    future::{ready, Ready},
    io::read_to_string,
    sync::{Arc, RwLock},
    path::PathBuf,
};

use actix_web::{
    error::{ErrorNotFound, ErrorUnauthorized},
    get, put,
    web::{Data, Path},
    App, Error as AError, FromRequest, HttpResponse, HttpServer, http::{self, header},
};
use clap::Parser;
use serde::Deserialize;

struct Auth;

impl FromRequest for Auth {
    type Error = AError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let config = match Data::<Config>::from_request(req, payload).into_inner() {
            Ok(c) => c,
            Err(e) => return ready(Err(e)),
        };

        if req.headers().get(header::AUTHORIZATION).map(|v| v.as_bytes().split(|c| *c == b' ').collect::<Vec<_>>()).map_or(true, |v| &[b"Bearer", config.password.as_bytes()] != v.as_slice()) {
            ready(Err(ErrorUnauthorized("unauthorized")))
        } else {
            ready(Ok(Self))
        }
    }
}

#[put("/{id}")]
async fn update_text(
    state: Data<State>,
    id: Path<String>,
    body: String,
    _: Auth,
) -> actix_web::Result<HttpResponse> {
    state.posts.write().unwrap().insert(id.into_inner(), body);
    println!("done {}", state.posts.read().unwrap().len());

    Ok(HttpResponse::Ok().finish())
}

#[get("/{id}")]
async fn get_text(
    state: Data<State>,
    id: Path<String>,
    _: Auth,
) -> actix_web::Result<String> {
    println!("{}", state.posts.read().unwrap().len());
    let Some(text) = state.posts.read().unwrap().get(&*id).cloned() else {
        return Err(ErrorNotFound("post not found"));
    };

    Ok(text)
}

#[derive(Default)]
struct State {
    posts: Arc<RwLock<HashMap<String, String>>>,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config: PathBuf,
}

#[derive(Deserialize)]
struct Config {
    bind_address: String,
    password: String,
}

#[actix_web::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let buf = read_to_string(File::open(&args.config).expect("failed to open config")).expect("failed to read config");
    let config: Config = toml::from_str(&buf).expect("failed to parse config");
    let config = Data::new(config);

    let state = Data::new(State::default());
    HttpServer::new({
        let config = config.clone();
        move || {
            App::new()
                .wrap(actix_cors::Cors::default().allowed_origin("https://zakru.fi").allowed_methods(vec!["GET"]).allowed_header(http::header::AUTHORIZATION).max_age(60))
                .wrap(actix_web::middleware::Logger::default())
                .app_data(config.clone())
                .app_data(state.clone())
                .app_data(actix_web::web::PayloadConfig::new(2 * 1024 * 1024))
                .service(update_text)
                .service(get_text)
        }
    })
    .bind(&config.bind_address)
    .expect("failed to bind")
    .run()
    .await
    .expect("server error");
}

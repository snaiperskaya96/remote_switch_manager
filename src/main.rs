use std::sync::Arc;

use argon2::password_hash::Salt;
use argon2::Argon2;
use argon2::PasswordHasher;
use axum::{
    extract::{Request, State}, http, middleware::{self, Next}, response::Response, routing::{get, post}, Json, Router
};
use futures_util::lock::Mutex;
use http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};

async fn auth(
    State(state): State<SafeAppState>,
    // run the `HeaderMap` extractor
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match get_token(&headers) {
        Some(token) if token_is_valid(token) => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

fn get_token(headers: &HeaderMap) -> Option<&str> {
    headers.get(AUTHORIZATION).and_then(|x| x.to_str().ok())
}

fn token_is_valid(token: &str) -> bool {    // ...

    false
}

#[derive(Serialize, Deserialize)]
struct Config {
    pub user_pass_hash: String,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let config_toml = std::env::current_exe().expect("Could not retrieve current_exe path").parent().expect("Could not retrieve parent's folder").join("config.toml");
        log::info!("Looking for {}", config_toml.display());
        if !std::path::Path::exists(&config_toml) {
            log::info!("{} does not exist, creating a new one.", config_toml.display());
            use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
            let hash = STANDARD_NO_PAD.encode(uuid::Uuid::new_v4());

            let toml_s = toml::to_string(&Self { user_pass_hash: hash }).expect("Could not turn Config struct into toml.");
            std::fs::write(&config_toml, toml_s).expect("Could not write to config.toml, check permissions");
        }
        return toml::from_str(&std::fs::read_to_string(&config_toml).expect("Could not read config.toml, make sure permissions are alright")).expect("Could not parse config.toml. Double check syntax and/or delete it.");
    }

    pub fn get_salt(&self) -> Result<Salt, argon2::password_hash::Error> {
        Salt::from_b64(&self.user_pass_hash)
    }
}

#[derive(Default)]
struct AppState {
    pub config: Config,
    pub users: Vec<User>
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct User {
    pub username: String,
    pub password: String,
}

// For toml serialization purposes
#[derive(PartialEq, Debug, Serialize, Deserialize, Default)]
struct UsersDb {
    pub users: Vec<User>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

impl AppState {
    pub fn init(&mut self) {
        let users_toml = std::env::current_exe().expect("Could not retrieve current_exe path").parent().expect("Could not retrieve parent's folder").join("users.toml");
        log::info!("Looking for {}", users_toml.display());

        if !std::path::Path::exists(&users_toml) {
            const DEFAULT_USER: &str = "admin";
            let new_pass = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

            log::info!("{} does not exist, creating a new one with username: {} and password: {}", users_toml.display(), DEFAULT_USER, new_pass);
            let a2 = Argon2::default();
            let hashed_pass = a2.hash_password(new_pass.as_bytes(), self.config.get_salt().expect("Could not retrieve salt from config")).expect("Could not hash new password");
            let users = UsersDb { users: vec![User::new(DEFAULT_USER.to_owned(), hashed_pass.serialize().to_string())] } ;
            
            let toml = toml::to_string(&users).unwrap();
            
            std::fs::write(&users_toml, &toml).expect("Could not write to users.toml, check permissions");
        }

        self.users = toml::from_str::<UsersDb>(&std::fs::read_to_string(users_toml).expect("Could not read users.toml, check permissions.")).expect("Could not convert toml to users array").users;
    }
}

type SafeAppState = Arc<Mutex<AppState>>;

#[tokio::main]
async fn main() {
    simplelog::TermLogger::init(
        log::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto
    ).expect("Unable to init termlogger");

    let mut state = AppState::default();
    state.init();
    // initialize tracing
    // tracing_subscriber::fmt::init();
    let state = Arc::new(Mutex::new(state));


    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .layer(middleware::from_fn_with_state(state.clone(), auth));
    
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

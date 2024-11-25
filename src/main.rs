use std::net::SocketAddr;
use std::sync::Arc;

use axum::{middleware::{self}, routing::get, Router};
use devices::{parse_devices_from_file, Switch};
use rand::distributions::{Alphanumeric, DistString};
use tokio::sync::RwLock;
use users::User;
use users::UsersDb;

pub mod config;
pub mod users;
pub mod auth;
pub mod devices;


#[derive(Default)]
pub struct AppState {
    pub config: config::Config,
    pub users: Vec<User>
}

impl AppState {
    pub fn init(&mut self) {
        let users_toml = std::env::current_exe().expect("Could not retrieve current_exe path").parent().expect("Could not retrieve parent's folder").join("users.toml");
        log::info!("Looking for {}", users_toml.display());

        if !std::path::Path::exists(&users_toml) {
            const DEFAULT_USER: &str = "admin";
            let new_pass = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

            log::info!("{} does not exist, creating a new one with username: {} and password: {}", users_toml.display(), DEFAULT_USER, new_pass);
            let users = UsersDb { users: vec![User::new(DEFAULT_USER.to_owned(), self.config.hash_password(&new_pass))] } ;
            
            let toml = toml::to_string(&users).unwrap();
            
            std::fs::write(&users_toml, &toml).expect("Could not write to users.toml, check permissions");
        }

        self.users = toml::from_str::<UsersDb>(&std::fs::read_to_string(users_toml).expect("Could not read users.toml, check permissions.")).expect("Could not convert toml to users array").users;
    }
}

pub type SafeAppState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    simplelog::TermLogger::init(
        log::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto
    ).expect("Unable to init termlogger");

    let devices = parse_devices_from_file();

    for dev in devices {
        dev.turn_off().await;
    }

    let mut state = AppState::default();
    state.init();
    let state = Arc::new(RwLock::new(state));

    let app = Router::new()
        .route("/", get(root))
        .with_state(state.clone())
        .merge(auth::add_auth_routes(state.clone()))
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

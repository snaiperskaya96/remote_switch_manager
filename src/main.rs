use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};
use devices::{parse_switches_from_file, Switch};
use rand::distributions::{Alphanumeric, DistString};
use timers::{parse_timers_from_file, Timer};
use tokio::sync::RwLock;
use users::parse_users_from_file;
use users::User;
use users::UsersDb;

pub mod config;
pub mod users;
pub mod auth;
pub mod devices;
pub mod timers;


#[derive(Default)]
pub struct AppState {
    pub config: config::Config,
    pub users: Vec<User>,
    pub switches: Vec<Box<dyn Switch>>,
    pub timers: Vec<Timer>,
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

    let mut state = AppState::default();
    state.switches = parse_switches_from_file();
    state.timers = parse_timers_from_file();
    state.users = parse_users_from_file(&state.config);

    let state = Arc::new(RwLock::new(state));

    {
        let state = state.clone();
        tokio::spawn(async move { timers::timers_task(state).await });
    }

    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/", get(root))
        .with_state(state.clone())
        .merge(auth::add_auth_routes(state.clone()))
        .merge(devices::add_devices_routes(state.clone()))
        .merge(timers::add_timers_routes(state.clone()))
        .layer(cors)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::auth_layer,
        ))
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8686").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    ""
}

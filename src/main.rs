use std::net::SocketAddr;
use std::sync::Arc;

use axum::response::{Html, IntoResponse, Response};
use axum::{routing::get, Router};
use devices::{parse_switches_from_file, Switch};
use http::{header, StatusCode, Uri};
use rust_embed::Embed;
use timers::{parse_timers_from_file, Timer};
use tokio::sync::RwLock;
use users::parse_users_from_file;
use users::User;

pub mod auth;
pub mod config;
pub mod devices;
pub mod timers;
pub mod users;

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
        simplelog::ColorChoice::Auto,
    )
    .expect("Unable to init termlogger");

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
        .route("/", get(index_handler))
        .route("/index.html", get(index_handler))
        .route("/assets/*file", get(static_handler))
        .with_state(state.clone())
        .merge(auth::add_auth_routes(state.clone()))
        .merge(devices::add_devices_routes(state.clone()))
        .merge(timers::add_timers_routes(state.clone()))
        .layer(cors)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::auth_layer,
        ))
        .fallback_service(get(not_found))
        .into_make_service_with_connect_info::<SocketAddr>();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8686").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> impl IntoResponse {
    static_handler("/index.html".parse::<Uri>().unwrap()).await
}

async fn not_found() -> Html<&'static str> {
    Html("<h1>404</h1><p>Not Found</p>")
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.starts_with("dist/") {
        path = path.replace("dist/", "");
    }

    StaticFile(path)
}

#[derive(Embed)]
#[folder = "public/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}

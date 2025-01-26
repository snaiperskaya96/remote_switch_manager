use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use http::{header::AUTHORIZATION, HeaderMap, Method, StatusCode};
use serde::{Deserialize, Serialize};

use crate::SafeAppState;

pub async fn auth_layer(
    State(state): State<SafeAppState>,
    // run the `HeaderMap` extractor
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    log::info!("Got request [{}] {} from {}",  request.method().as_str(), request.uri().path(), addr.to_string());
    if !request.uri().path().starts_with("/api/") || request.method() == Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    match get_token(&headers) {
        Some(token) => {
            if token.starts_with("Basic ") {
                let b64 = token.split_at("Basic ".len()).1;
                use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
                let credentials = STANDARD_NO_PAD.decode(b64).unwrap_or(Vec::new());
                let parts = credentials.split_at(credentials.iter().enumerate().find(|x| *x.1 == ':' as u8).unwrap_or((0, &0)).0);

                let user = String::from_utf8(parts.0.to_vec()).unwrap_or("".to_owned());
                let pass = String::from_utf8(parts.1.to_vec()).unwrap_or("".to_owned());

                if let Some(_) = get_user_by_credentials(state.clone(), &user, &pass).await {
                    Ok(next.run(request).await)
                } else {
                    log::info!(
                        "Unauthorized request to uri {} from client {}. No matching token found for basic auth {}",
                        request.uri(),
                        addr.to_string(),
                        b64
                    );
                    Err(StatusCode::UNAUTHORIZED)
                }
            } else if token_is_valid(&state, token).await {
                // refresh token ?
                Ok(next.run(request).await)
            } else {
                log::info!(
                    "Unauthorized request to uri {} from client {}. No matching token found for {}",
                    request.uri(),
                    addr.to_string(),
                    token
                );
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            log::info!(
                "Unauthorized request to uri {} from client {}, no token header found.",
                request.uri(),
                addr.to_string()
            );
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

fn get_token(headers: &HeaderMap) -> Option<&str> {
    headers.get(AUTHORIZATION).and_then(|x| x.to_str().ok())
}

async fn token_is_valid(state: &SafeAppState, token: &str) -> bool {
    let lock = state.read().await;
    for user in &lock.users {
        if let Some(token_expiry) = user.auth_tokens.get(token) {
            if token_expiry.elapsed().as_secs() < lock.config.user_token_expiry_time_seconds {
                return true;
            }
        }
    }
    false
}

#[derive(Deserialize)]
pub struct SignInRequest {
    username: String,
    password: String,
}
#[derive(Serialize)]
pub struct SignInResponse {
    success: bool,
    token: Option<String>,
}

async fn get_user_by_credentials(state: SafeAppState, username: &String, password: &String) -> Option<usize> {
    let lock = state.read().await;
    for (id, user) in lock.users.iter().enumerate() {
        if user.username == *username {
            if lock
                .config
                .verify_password(&password, &user.password)
            {
                return Some(id);
            }
            break;
        }
    }

    None
}

pub async fn sign_in(
    State(state): State<SafeAppState>,
    Json(payload): Json<SignInRequest>,
) -> Result<Json<SignInResponse>, (StatusCode, String)> {
    let user_ref = get_user_by_credentials(state.clone(), &payload.username, &payload.password).await;
    let expiry_time;

    {
        let lock = state.read().await;
        expiry_time = tokio::time::Instant::now()
        .checked_add(tokio::time::Duration::from_secs(
            lock.config.user_token_expiry_time_seconds,
        ))
        .unwrap();
    }

    if let Some(logged_in_user) = user_ref {
        let mut lock = state.write().await;
        let user = lock
            .users
            .get_mut(logged_in_user)
            .expect("Could not find user that was just password verified");
        let new_token = uuid::Uuid::new_v4().to_string();
        user.auth_tokens.insert(new_token.clone(), expiry_time);
        return Ok(Json(SignInResponse { success: true, token: Some(new_token) }));
    }

    Ok(Json(SignInResponse { success: false, token: None }))
}

pub async fn is_logged_in(
    headers: HeaderMap,
    State(state): State<SafeAppState>,
) -> Result<(), (StatusCode, String)> {
    match get_token(&headers) {
        Some(token) => {
            if token_is_valid(&state, token).await {
                Ok(())
            } else {
                Err((StatusCode::UNAUTHORIZED, "Invalid Auth Token".to_owned()))
            }
        },
        None => Err((StatusCode::UNAUTHORIZED, "Invalid Auth Token".to_owned())),
    }
}

pub fn add_auth_routes(state: SafeAppState) -> Router {
    Router::new()
        .route("/sign_in", post(sign_in))
        .route("/logged_in", get(is_logged_in))
        // .layer(axum::middleware::from_fn_with_state(
        //     state.clone(),
        //     auth_layer,
        // ))
        .with_state(state)
}


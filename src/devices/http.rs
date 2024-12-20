use axum::{extract::{Path, State}, routing::get, Json, Router};
use http::StatusCode;
use serde::Serialize;

use crate::SafeAppState;

use super::DeviceDataSafe;

async fn get_switches(
    State(state): State<SafeAppState>,
) -> Result<Json<Vec<DeviceDataSafe>>, (StatusCode, String)> {
    let mut switches: Vec<DeviceDataSafe> = Vec::new();

    let lock = state.read().await;
    for switch in &lock.switches {
        switches.push(switch.get_device_data().into());
    }

    Ok(Json(switches))
}

#[derive(Serialize)]
 struct TurnOnOffResponse { success: bool }

 async fn turn_on(
    State(state): State<SafeAppState>,
    Path(id): Path<u32>,
) -> Result<Json<TurnOnOffResponse>, (StatusCode, String)> {
    let mut lock = state.write().await;

    match lock.switches.iter_mut().find(|x| x.get_device_data().id == id) {
        Some(switch) => {
            switch.turn_on().await;
            Ok(Json(TurnOnOffResponse { success: true}))
        },
        None => Err((StatusCode::BAD_REQUEST, "Could not find any switch with the given id".to_owned())),
    }
}

async fn turn_off(
    State(state): State<SafeAppState>,
    Path(id): Path<u32>,
) -> Result<Json<TurnOnOffResponse>, (StatusCode, String)> {
    let mut lock = state.write().await;

    match lock.switches.iter_mut().find(|x| x.get_device_data().id == id) {
        Some(switch) => {
            switch.turn_off().await;
            Ok(Json(TurnOnOffResponse { success: true}))
        },
        None => Err((StatusCode::BAD_REQUEST, "Could not find any switch with the given id".to_owned())),
    }
}

pub fn add_devices_routes(state: SafeAppState) -> Router {
    Router::new()
        .route("/api/switches", get(get_switches))
        .route("/api/turn_on/:id", get(turn_on))
        .route("/api/turn_off/:id", get(turn_off))
        .with_state(state)
}


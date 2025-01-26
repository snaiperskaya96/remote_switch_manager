use axum::{extract::{Path, State}, routing::{get, post}, Json, Router};
use http::StatusCode;
use serde::Serialize;

use crate::SafeAppState;

use super::{store_timers, Timer};

pub async fn get_device_timers(
    State(state): State<SafeAppState>,
    Path(id): Path<u32>,
) -> Result<Json<Vec<Timer>>, (StatusCode, String)> {
    let mut out = Vec::new();

    for timer in &state.read().await.timers {
        if timer.switch_id == id {
            out.push(timer.clone());
        }
    }

    Ok(Json(out))
}

#[derive(Serialize)]
struct AddTimerResponse { success: bool }

async fn add_timer(
    State(state): State<SafeAppState>,
    Json(mut timer): Json<Timer> 
) -> Result<Json<AddTimerResponse>, (StatusCode, String)>
{
    let mut lock = state.write().await;
    
    let new_id = make_timer_id(&lock.timers);

    timer.id = new_id;

    lock.timers.push(timer);

    store_timers(&lock.timers);

    Ok(Json(AddTimerResponse { success: true }))
}

fn make_timer_id(timers: &Vec<Timer>) -> u32 {
    let mut id = 0;

    for timer in timers {
        id = timer.id.max(id);
    }

    return id + 1;
}

pub fn add_timers_routes(state: SafeAppState) -> Router {
    Router::new()
        .route("/api/timers/{id}", get(get_device_timers))
        .route("/api/timer", post(add_timer))
        .with_state(state)
}


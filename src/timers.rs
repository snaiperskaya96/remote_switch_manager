use axum::{extract::{Path, State}, routing::{get, post}, Json, Router};
use chrono::Datelike;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{devices::DeviceStatus, SafeAppState};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub id: u32,      
    pub switch_id: u32,    
    pub start_time: u32, // Start time in minutes after midnight
    pub end_time: u32,   // End time in minutes after midnight
    pub days: Vec<u8>,   // Array of days (0=Sunday, 6=Saturday)
    pub is_active: bool,  
}

impl Timer {
    pub fn should_be_active(&self) -> bool {
        let minutes_since_midnight = Timer::minutes_since_midnight();
        let now = chrono::Local::now();
        let matches_day = self.days.contains(&(now.weekday().num_days_from_monday() as u8));
        matches_day && minutes_since_midnight >= self.start_time.into() && minutes_since_midnight <= self.end_time.into()
    }

    fn minutes_since_midnight() -> u32 {
        let now = chrono::Local::now();

        let tomorrow_midnight = now.date_naive().and_hms_opt(0, 0, 1).unwrap();
    
        let duration = now.naive_local().signed_duration_since(tomorrow_midnight).to_std().unwrap();
        (duration.as_secs() / 60) as _
    }
}

#[derive(Deserialize, Serialize)]
struct TimersArray {
    timers: Vec<Timer>,
}


pub fn store_timers(timers: &Vec<Timer>) {
    let timers_toml = std::env::current_exe()
    .expect("Could not retrieve current_exe path")
    .parent()
    .expect("Could not retrieve parent's folder")
    .join("timers.toml");
    log::info!("Storing timers into {}", timers_toml.display());

    std::fs::write(timers_toml, toml::to_string(&TimersArray { timers: timers.clone() }).expect("Could not serialize timers array.")).expect("Could not write to timers.toml, check permissions.");
}

pub fn parse_timers_from_file() -> Vec<Timer> {
    let timers_toml = std::env::current_exe()
        .expect("Could not retrieve current_exe path")
        .parent()
        .expect("Could not retrieve parent's folder")
        .join("timers.toml");
    log::info!("Looking for {}", timers_toml.display());

    let mut out = Vec::new();

    if std::path::Path::exists(&timers_toml) {
        log::info!("Parsing timers.toml");
        let switches_str = std::fs::read_to_string(timers_toml)
            .expect("Unable to parse timers.toml. Check permissions.");
        out = toml::from_str::<TimersArray>(&switches_str)
            .expect("Unable to paese timers.toml content")
            .timers;
        log::info!("Parsed {} timers.", out.len());
    } else {
        log::info!("No switches.toml found.");
    }

    out
}


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
        .route("/api/timers/:id", get(get_device_timers))
        .route("/api/timer", post(add_timer))
        .with_state(state)
}

pub async fn timers_task(state: SafeAppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        {
            let mut lock = state.write().await;
            let config = &mut *lock;
            let timers = &mut config.timers;
            let switches = &mut config.switches;

            for switch in &mut *switches {
                let switch_data = switch.get_device_data().clone();
                for timer in &mut *timers {
                    if !timer.is_active || timer.switch_id != switch_data.id {
                        continue;
                    }

                    let should_be_active = timer.should_be_active();
                    let current_switch_status = switch_data.status.as_ref().unwrap_or(&DeviceStatus::Unknown);

                    if should_be_active && current_switch_status == &DeviceStatus::Off {
                        log::info!("Turning on switch {} for timer {}", switch_data.alias, timer.id);
                        switch.turn_on().await;
                    } else if !should_be_active && current_switch_status == &DeviceStatus::On {
                        log::info!("Turning off switch {} for timer {}", switch_data.alias, timer.id);
                        switch.turn_off().await;
                    }
                }
            }
        }
        interval.tick().await;
    }
}
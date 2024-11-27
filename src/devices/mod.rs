use axum::{extract::{Path, State}, routing::get, Json, Router};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use shelly::ShellySwitch;

use crate::SafeAppState;

pub mod shelly;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum Device {
    Shelly,
}


#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum DeviceStatus {
    On,
    Off,
    Unknown,
}

pub trait Switch : Send + Sync {
    fn turn_on(&mut self) -> futures::future::BoxFuture<()>;
    fn turn_off(&mut self) -> futures::future::BoxFuture<()>;
    fn serialize(&self) -> String;
    fn get_device_data(&self) -> &DeviceData;
}

/*
* This is meant to be sent to frontend without including sensitive information
*/
#[derive(Serialize)]
pub struct DeviceDataSafe
{
    alias: String,
    id: u32,
    #[serde(alias = "type")]
    device_type: Device,
    status: Option<DeviceStatus>,
}

impl From<&DeviceData> for DeviceDataSafe {
    fn from(value: &DeviceData) -> Self {
        Self { alias: value.alias.clone(), id: value.id, device_type: value.device_type.clone(), status: value.status.clone() }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceData {
    pub alias: String,
    pub addr: String,
    pub id: u32,
    pub username: String,
    pub password: String,
    #[serde(alias = "type")]
    pub device_type: Device,
    pub status: Option<DeviceStatus>,
}

pub fn parse_switches_from_file() -> Vec<Box<dyn Switch>> {
    let switches_toml = std::env::current_exe()
        .expect("Could not retrieve current_exe path")
        .parent()
        .expect("Could not retrieve parent's folder")
        .join("switches.toml");
    log::info!("Looking for {}", switches_toml.display());

    let mut out = Vec::new();

    if std::path::Path::exists(&switches_toml) {
        #[derive(Deserialize)]
        struct SwitchesArray {
            switches: Vec<DeviceData>,
        }

        log::info!("Parsing switches.toml");
        let switches_str = std::fs::read_to_string(switches_toml)
            .expect("Unable to parse switches.toml. Check permissions.");
        let switches_array: Vec<DeviceData> = toml::from_str::<SwitchesArray>(&switches_str)
            .expect("Unable to paese switches.toml content")
            .switches;
        log::info!("Parsed {} switches.", switches_array.len());

        for switch_data in switches_array {
            out.push(create_switch_from_data(switch_data));
        }
    } else {
        log::info!("No switches.toml found.");
    }

    out
}

fn create_switch_from_data(device_data: DeviceData) -> Box<dyn Switch> {
    match device_data.device_type {
        Device::Shelly => Box::new(ShellySwitch::new(
            device_data.alias,
            device_data.addr,
            device_data.id,
            device_data.username,
            device_data.password,
        )),
    }
}

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
        .route("/switches", get(get_switches))
        .route("/turn_on/:id", get(turn_on))
        .route("/turn_off/:id", get(turn_off))
        .with_state(state)
}

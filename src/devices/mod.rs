use serde::{Deserialize, Serialize};
use shelly::ShellySwitch;

use crate::SafeAppState;

pub mod shelly;
pub mod http;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Device {
    Shelly,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum DeviceStatus {
    On,
    Off,
    Unknown,
}

pub trait Switch : Send + Sync {
    fn turn_on(&mut self) -> futures::future::BoxFuture<()>;
    fn turn_off(&mut self) -> futures::future::BoxFuture<()>;
    fn update_status(&mut self) -> futures::future::BoxFuture<()>;
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

#[derive(Serialize, Deserialize, Clone, Debug)]
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

pub async fn devices_status_task(state: SafeAppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));


    loop {
        let num_switches = state.read().await.switches.len();
        for i in 0..num_switches {
            let mut lock = state.write().await;
            lock.switches.get_mut(i).unwrap().update_status().await;
        }
        interval.tick().await;
    }
}

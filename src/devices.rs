use diqwest::WithDigestAuth;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum Device {
    Shelly,
}

pub trait Switch {
    fn turn_on(&self) -> impl std::future::Future<Output = ()> + Send;
    fn turn_off(&self) -> impl std::future::Future<Output = ()> + Send;
    fn serialize(&self) -> String;
}

#[derive(Serialize, Deserialize)]
pub struct DeviceData {
    alias: String,
    addr: String,
    id: u64,
    username: String,
    password: String,
    #[serde(alias="type")]
    device_type: Device,
}

pub struct ShellySwitch {
    data: DeviceData,
    client: reqwest::Client,
}

unsafe impl Send for ShellySwitch {}

pub fn parse_devices_from_file() -> Vec<Box<impl Switch>> {
    let switches_toml = std::env::current_exe().expect("Could not retrieve current_exe path").parent().expect("Could not retrieve parent's folder").join("switches.toml");
    log::info!("Looking for {}", switches_toml.display());

    let mut out = Vec::new();

    if std::path::Path::exists(&switches_toml) {
        #[derive(Deserialize)]
        struct SwitchesArray { switches: Vec<DeviceData> }
       
        log::info!("Parsing switches.toml");
        let switches_str = std::fs::read_to_string(switches_toml).expect("Unable to parse switches.toml. Check permissions.");
        let switches_array: Vec<DeviceData> = toml::from_str::<SwitchesArray>(&switches_str).expect("Unable to paese switches.toml content").switches;
        log::info!("Parsed {} switches.", switches_array.len());

        for switch_data in switches_array {
            out.push(create_from_data(switch_data));
        }
    } else {
        log::info!("No switches.toml found.");
    }

    out
}

fn create_from_data(device_data: DeviceData) -> Box<impl Switch> {
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

impl ShellySwitch {
    pub fn new(alias: String, addr: String, id: u64, username: String, password: String) -> Self {
        Self {
            data: DeviceData {
                alias,
                addr,
                id,
                username,
                password,
                device_type: Device::Shelly,
            },
            client: reqwest::Client::new(),
        }
    }
}

impl Switch for ShellySwitch {
    fn turn_on(&self) -> impl std::future::Future<Output = ()> + Send {
        async move {
            self.client
                .get(format!(
                    "http://{}/rpc/Switch.Set?id={}&on=true",
                    self.data.addr, self.data.id
                ))
                .send_with_digest_auth(&self.data.username, &self.data.password)
                .await
                .unwrap();
        }
    }

    fn turn_off(&self) -> impl std::future::Future<Output = ()> + Send {
        async move {
            self.client
                .get(format!(
                    "http://{}/rpc/Switch.Set?id={}&on=false",
                    self.data.addr, self.data.id
                ))
                .send_with_digest_auth(&self.data.username, &self.data.password)
                .await
                .unwrap();
        }
    }
    
    fn serialize(&self) -> String {
        toml::to_string(&self.data).unwrap()
    }
}

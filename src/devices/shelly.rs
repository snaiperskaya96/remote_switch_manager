use diqwest::WithDigestAuth;
use serde::Deserialize;
use serde::Serialize;

use super::Device;
use super::DeviceData;
use super::DeviceStatus;
use super::Switch;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusResponse {
    pub id: i64,
    pub source: String,
    pub output: bool,
    pub apower: f64,
    pub voltage: f64,
    pub current: f64,
    pub aenergy: GetStatusResponseEnergy,
    pub temperature: GetStatusTemperature,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusResponseEnergy {
    pub total: f64,
    #[serde(rename = "by_minute")]
    pub by_minute: Vec<f64>,
    #[serde(rename = "minute_ts")]
    pub minute_ts: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusTemperature {
    pub t_c: f64,
    pub t_f: f64,
}

#[derive(Debug)]
pub struct ShellySwitch {
    data: DeviceData,
    client: reqwest::Client,
}

unsafe impl Send for ShellySwitch {}


impl ShellySwitch {
    pub fn new(alias: String, addr: String, id: u32, username: String, password: String) -> Self {
        let mut s = Self {
            data: DeviceData {
                alias,
                addr,
                id,
                username,
                password,
                device_type: Device::Shelly,
                status: None,
            },
            client: reqwest::Client::new(),
        };

        futures::executor::block_on(s.update_status());

        s
    }
}

impl Switch for ShellySwitch {
    fn turn_on(&mut self) -> futures::future::BoxFuture<()> {
        Box::pin(async move {
            match self.client
                .get(format!(
                    "http://{}/rpc/Switch.Set?id={}&on=true",
                    self.data.addr, self.data.id
                ))
                .send_with_digest_auth(&self.data.username, &self.data.password)
                .await {
                    Ok(_r) => { self.data.status = Some(DeviceStatus::On); },
                    Err(e) => { log::warn!("There was an error while trying to turn on shelly {}: {:?}", self.data.alias, e)},
                }
        })
    }

    fn turn_off(&mut self) -> futures::future::BoxFuture<()> {
        Box::pin(async move {
            match self.client
                .get(format!(
                    "http://{}/rpc/Switch.Set?id={}&on=false",
                    self.data.addr, self.data.id
                ))
                .send_with_digest_auth(&self.data.username, &self.data.password)
                .await {
                    Ok(_r) => { self.data.status = Some(DeviceStatus::Off); },
                    Err(e) => { log::warn!("There was an error while trying to turn off shelly {}: {:?}", self.data.alias, e)},
                }
        })
    }

    fn serialize(&self) -> String {
        toml::to_string(&self.data).unwrap()
    }
    
    fn get_device_data(&self) -> &DeviceData {
        &self.data
    }
    
    fn update_status(&mut self) -> futures::future::BoxFuture<()> {
        Box::pin(async {
            log::debug!("Checking switch {}'s status", self.data.alias);
            if let Ok(res) = self.client
            .get(format!(
                "http://{}/rpc/Switch.GetStatus?id={}",
                self.data.addr, self.data.id
            ))
            .send_with_digest_auth(&self.data.username, &self.data.password)
            .await {
                match res.json::<GetStatusResponse>().await {
                    Ok(status) => {
                        if status.output {
                            self.data.status = Some(super::DeviceStatus::On);
                        } else {
                            self.data.status = Some(super::DeviceStatus::Off);
                        }
                    },
                    Err(e) => {
                        log::warn!("There was an error while retrieving switch {}'s status: {:?}", self.data.alias, e);
                    },
                }
            }
        })
    }
}


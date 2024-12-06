use chrono::{DateTime, Datelike, Local, Utc};
use serde::{Deserialize, Serialize};
use chrono_tz::Tz;

use crate::{devices::DeviceStatus, storage::get_storage_path, SafeAppState};

pub mod http;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub id: u32,      
    pub switch_id: u32,    
    pub start_time: u32, // Start time in minutes after midnight
    pub end_time: u32,   // End time in minutes after midnight
    pub days: Vec<u8>,   // Array of days (0=Sunday, 6=Saturday)
    pub is_active: bool,  
    pub one_off: bool,
}

impl Timer {
    fn now(timezone: &Option<String>) -> DateTime<Tz> {
        match timezone {
            Some(tz_name) => {
                let tz: Tz = tz_name
                    .parse()
                    .expect("Invalid timezone name. Use a valid IANA timezone, e.g., 'Europe/Rome'.");
                
                let now_utc: DateTime<Utc> = Utc::now();
                now_utc.with_timezone(&tz)
            }
            None => {
                let local_time = Local::now();

                let tz_name = iana_time_zone::get_timezone()
                    .expect("Could not determine timezone.");

                let tz: Tz = tz_name
                .parse()
                .expect("Invalid timezone name.");

                // Convrt the local time to UTC
                local_time.with_timezone(&tz)
            }
        }
    }
    
    pub fn should_be_on(&self, timezone_override: &Option<String>) -> bool {
        let minutes_since_midnight = Timer::minutes_since_midnight(timezone_override);
        let now = Self::now(timezone_override);
        let matches_day = self.days.contains(&(now.weekday().num_days_from_monday() as u8));
        matches_day && minutes_since_midnight >= self.start_time.into() && minutes_since_midnight <= self.end_time.into()
    }

    fn minutes_since_midnight(timezone_override: &Option<String>) -> u32 {
        let now = Timer::now(timezone_override);

        let tomorrow_midnight = now.date_naive().and_hms_opt(0, 0, 1).unwrap();
    
        let duration = now.naive_local().signed_duration_since(tomorrow_midnight).to_std().unwrap();
        (duration.as_secs() / 60) as _
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

#[derive(Deserialize, Serialize)]
struct TimersArray {
    timers: Vec<Timer>,
}

pub fn parse_timers_from_file() -> Vec<Timer> {
    let timers_toml = get_storage_path().join("timers.toml");
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

pub fn store_timers(timers: &Vec<Timer>) {
    let timers_toml = get_storage_path().join("timers.toml");
    log::info!("Storing timers into {}", timers_toml.display());

    std::fs::write(timers_toml, toml::to_string(&TimersArray { timers: timers.clone() }).expect("Could not serialize timers array.")).expect("Could not write to timers.toml, check permissions.");
}


pub async fn timers_task(state: SafeAppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        {
            let mut lock = state.write().await;
            let state = &mut *lock;
            let timers = &mut state.timers;
            let switches = &mut state.switches;

            let mut any_timer_changed = false;

            for switch in &mut *switches {
                let switch_data = switch.get_device_data().clone();
                for timer in &mut *timers {
                    if !timer.is_active || timer.switch_id != switch_data.id {
                        continue;
                    }

                    let should_be_on = timer.should_be_on(&state.config.timezone_override);
                    let current_switch_status = switch_data.status.as_ref().unwrap_or(&DeviceStatus::Unknown);

                    if should_be_on && current_switch_status == &DeviceStatus::Off {
                        log::info!("Turning on switch {} because of timer {}", switch_data.alias, timer.id);
                        switch.turn_on().await;
                    } else if !should_be_on && current_switch_status == &DeviceStatus::On {
                        log::info!("Turning off switch {} because of timer {}", switch_data.alias, timer.id);
                        switch.turn_off().await;

                        if timer.one_off {
                            timer.deactivate();
                            any_timer_changed = true;
                        }
                    }
                }
            }

            if any_timer_changed {
                store_timers(&state.timers);
            }
        }

        interval.tick().await;
    }
}
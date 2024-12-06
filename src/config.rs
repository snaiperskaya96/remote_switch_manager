use argon2::{
    password_hash::{PasswordHashString, Salt},
    Argon2, PasswordHasher, PasswordVerifier,
};
use serde::{Deserialize, Serialize};

use crate::storage::get_storage_path;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub user_pass_hash: String,
    pub user_token_expiry_time_seconds: u64,
    // to overcome musl (i guess?) bug where local timezone is ignored
    pub timezone_override: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        let config_toml = get_storage_path().join("config.toml");
        log::info!("Looking for {}", config_toml.display());
        if !std::path::Path::exists(&config_toml) {
            log::info!(
                "{} does not exist, creating a new one.",
                config_toml.display()
            );
            use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine as _};
            let hash = STANDARD_NO_PAD.encode(uuid::Uuid::new_v4());

            let toml_s = toml::to_string(&Self {
                user_pass_hash: hash,
                user_token_expiry_time_seconds: 60 * 60 * 24 * 7,
                ..Default::default()
            })
            .expect("Could not turn Config struct into toml.");
            std::fs::write(&config_toml, toml_s)
                .expect("Could not write to config.toml, check permissions");
        }
        return toml::from_str(
            &std::fs::read_to_string(&config_toml)
                .expect("Could not read config.toml, make sure permissions are alright"),
        )
        .expect("Could not parse config.toml. Double check syntax and/or delete it.");
    }

    pub fn get_salt(&self) -> Result<Salt, argon2::password_hash::Error> {
        Salt::from_b64(&self.user_pass_hash)
    }

    pub fn hash_password(&self, password: &String) -> String {
        let a2 = Argon2::default();
        let hashed_pass = a2
            .hash_password(
                password.as_bytes(),
                self.get_salt()
                    .expect("Could not retrieve salt from config"),
            )
            .expect("Could not hash new password");
        hashed_pass.serialize().to_string()
    }

    pub fn verify_password(&self, plain_password: &String, hash: &String) -> bool {
        let a2 = Argon2::default();
        match a2.verify_password(
            plain_password.as_bytes(),
            &PasswordHashString::parse(hash, argon2::password_hash::Encoding::B64)
                .expect("Could not parse password hash")
                .password_hash(),
        ) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

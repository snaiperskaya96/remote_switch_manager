use std::collections::HashMap;

use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};

use crate::{config::Config, storage::get_storage_path};

#[derive(PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
    // token / expiry
    #[serde(skip)]
    pub auth_tokens: HashMap<String, tokio::time::Instant>
}

// For toml serialization purposes
#[derive(PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct UsersDb {
    pub users: Vec<User>,
}

impl User {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password, ..Self::default() }
    }
}

pub fn parse_users_from_file(config: &Config) -> Vec<User> {
    let users_toml: std::path::PathBuf = get_storage_path().join("users.toml");
    log::info!("Looking for {}", users_toml.display());

    if !std::path::Path::exists(&users_toml) {
        const DEFAULT_USER: &str = "admin";
        let new_pass = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        log::info!("{} does not exist, creating a new one with username: {} and password: {}", users_toml.display(), DEFAULT_USER, new_pass);
        let users = UsersDb { users: vec![User::new(DEFAULT_USER.to_owned(), config.hash_password(&new_pass))] } ;
        
        let toml = toml::to_string(&users).unwrap();
        
        std::fs::write(&users_toml, &toml).expect("Could not write to users.toml, check permissions");
    }

    toml::from_str::<UsersDb>(&std::fs::read_to_string(users_toml).expect("Could not read users.toml, check permissions.")).expect("Could not convert toml to users array").users
}

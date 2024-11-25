use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

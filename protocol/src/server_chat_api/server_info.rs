use serde::{Deserialize, Serialize};

pub const SERVER_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, PartialOrd)]
pub struct ServerInfo {
    pub server_version: i64,
    pub server_name: String,
}
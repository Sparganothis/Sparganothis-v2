use std::time::Duration;

pub const GLOBAL_CHAT_TOPIC_ID: &'static str = "global";

pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(7);
pub const PRESENCE_IDLE: Duration = Duration::from_secs(16);
pub const PRESENCE_EXPIRATION: Duration = Duration::from_secs(30);
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
pub const GLOBAL_PERIODIC_TASK_INTERVAL: Duration = Duration::from_secs(5);


pub const IROH_RELAY_DOMAIN: &str = "127.0.0.1";
use std::time::Duration;

pub const GLOBAL_CHAT_TOPIC_ID: &'static str = "global";

pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(4);
pub const PRESENCE_IDLE: Duration = Duration::from_secs(25);
pub const PRESENCE_EXPIRATION: Duration = Duration::from_secs(60);
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
pub const GLOBAL_PERIODIC_TASK_INTERVAL: Duration = Duration::from_secs(5);

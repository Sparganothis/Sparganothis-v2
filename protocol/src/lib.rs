use chrono::{DateTime, Utc};

pub(crate) mod _bootstrap_keys;
pub mod _const;
pub(crate) mod _random_word;
pub mod chat;
pub mod chat_presence;
pub mod chat_ticket;
pub(crate) mod echo;
pub mod game_matchmaker;
pub mod global_chat;
pub mod global_matchmaker;
pub(crate) mod main_node;
pub mod server_chat_api;
pub(crate) mod signed_message;
pub(crate) mod sleep;
pub mod user_identity;

pub fn timestamp_micros() -> u128 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

pub fn datetime_now() -> DateTime<Utc> {
    let timestamp = timestamp_micros() as i64;
    DateTime::<Utc>::from_timestamp_micros(timestamp).unwrap()
}

pub use signed_message::*;
pub use paste;
pub use postcard;
pub use inventory;
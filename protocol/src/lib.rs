use chrono::{DateTime, Utc};

pub(crate) mod _bootstrap_keys;
pub(crate) mod _random_word;
pub mod api;
pub mod chat;
pub(crate) mod echo;
pub mod global_matchmaker;
pub(crate) mod main_node;
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

pub use paste;
pub use postcard;
pub use signed_message::*;

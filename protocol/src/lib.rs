pub mod _const;
pub mod _random_word;
pub mod chat;
pub mod chat_presence;
pub mod echo;
pub mod global_matchmaker;
pub mod main_node;
pub mod sleep;
pub mod user_identity;
pub mod _matchbox_signal;

pub(crate) mod _bootstrap_keys;


pub fn get_timestamp() -> u128 {
    web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).unwrap().as_micros()
}
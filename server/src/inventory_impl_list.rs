// #[macro_export]
macro_rules! api_wrapper_fn {
    ($name: tt) => { $crate::paste::paste! {
         [< __ $name _wrapper3>] 
    }
}}

// #[macro_export]
macro_rules! api_method_impl {
    ($name: tt) => { $crate::paste::paste! {
        ApiMethodImpl {
            name: stringify!($name),
            func: api_wrapper_fn!($name),
        }
    }
}}

use protocol::server_chat_api::api_method_macros::ApiMethodImpl;

use crate::server::db::get_replay_match_list2::*;
use crate::server::db::guest_login::*;
use crate::server::db::send_new_gamestate::*;
use crate::server::db::send_new_match::*;

pub const INVENTORY_FUNCTIONS_IMPL: [ApiMethodImpl; 6] = [

    /*                         get_replay_match_list2           */
    /* ======================================================== */
    api_method_impl!(GetReplayMatchList),
    api_method_impl!(GetReplayMatchDetail),
    api_method_impl!(GetGameStateRowsForMatch),


    /*                         get_replay_match_list2           */
    /* ======================================================== */
    api_method_impl!(LoginApiMethod),

    
    /*                         send_new_gamestate           */
    /* ======================================================== */
    api_method_impl!(SendNewGameState),

    /*                         send_new_match           */
    /* ======================================================== */
    api_method_impl!(SendNewMatch),

];

pub fn inventory_get_implementation_by_name(
    name: &str,
) -> anyhow::Result<&'static ApiMethodImpl> {
    for x in &INVENTORY_FUNCTIONS_IMPL {
        if x.name == name {
            return Ok(x);
        }
    }
    anyhow::bail!("method not found!")
}
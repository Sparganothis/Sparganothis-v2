// #[macro_export]
macro_rules! api_wrapper_fn {
    ($name: tt) => {
        $crate::paste::paste! {
             [< __ $name _wrapper3>]
        }
    };
}

// #[macro_export]
macro_rules! api_method_impl {
    ($name: tt) => {
        $crate::paste::paste! {
            ApiMethodImpl {
                name: stringify!($name),
                func: api_wrapper_fn!($name),
            }
        }
    };
}

use protocol::impl_api_method;
use protocol::server_chat_api::api_method_macros::ApiMethodImpl;

use crate::server::db::get_replay_match_list2::*;
use crate::server::db::guest_login::*;
use crate::server::db::send_new_gamestate::*;
use crate::server::db::send_new_match::*;
use crate::server::multiplayer::matchmaker::matchmaker_api::*;

use protocol::server_chat_api::api_declarations::*;

impl_api_method!(GetReplayMatchList, db_get_list_matches); // INVENTORY OK
impl_api_method!(GetReplayMatchDetail, db_get_detail_match); // INVENTORY OK
impl_api_method!(GetGameStateRowsForMatch, db_get_game_states_for_match); // inventory ok
impl_api_method!(SendNewGameState, db_send_new_gamestate);
impl_api_method!(SendNewMatch, db_send_new_match);
impl_api_method!(LoginApiMethod, db_add_guest_login); // inventory ok
impl_api_method!(RunMultiplayerMatchmakerPhase1, run_multiplayer_matchmaker_1);
impl_api_method!(RunMultiplayerMatchmakerPhase2, run_multiplayer_matchmaker_2);

pub const INVENTORY_FUNCTIONS_IMPL: [ApiMethodImpl; 8] = [
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

    /*                         matchmaker_api           */
    /* ======================================================== */
    api_method_impl!(RunMultiplayerMatchmakerPhase1),
    api_method_impl!(RunMultiplayerMatchmakerPhase2),
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

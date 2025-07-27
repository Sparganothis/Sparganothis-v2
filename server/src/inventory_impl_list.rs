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

use protocol::api::api_method_macros::ApiMethodImpl;
use protocol::impl_api_method;

use crate::server::db::get_replay_match_list2::*;
use crate::server::db::get_user_profiles::*;
use crate::server::db::guest_login::*;
use crate::server::db::send_new_gamestate::*;
use crate::server::db::send_new_match::*;
use crate::server::db::user_friends::*;
use crate::server::multiplayer::matchmaker::matchmaker_api::*;
use protocol::api::api_declarations::*;

impl_api_method!(GetReplayMatchList, db_get_list_matches); // INVENTORY OK
impl_api_method!(GetReplayMatchDetail, db_get_detail_match); // INVENTORY OK
impl_api_method!(GetGameStateRowsForMatch, db_get_game_states_for_match); // inventory ok

impl_api_method!(GetLastGameStatesForMatch, get_last_game_states_for_match);
impl_api_method!(SendNewGameState, db_send_new_gamestate);
impl_api_method!(SendNewMatch, db_send_new_match);
impl_api_method!(LoginApiMethod, db_add_guest_login); // inventory ok
                                                      //  ======================= multiplayer ====================================
impl_api_method!(RunMultiplayerMatchmakerPhase1, run_multiplayer_matchmaker_1);
impl_api_method!(RunMultiplayerMatchmakerPhase2, run_multiplayer_matchmaker_2);
//  ======================= user_friends ====================================
impl_api_method!(UserAddFriend, user_add_friend);
impl_api_method!(UserDeleteFriend, user_delete_friend);
impl_api_method!(UserGetFriends, user_list_friends);
// ================== user_profiles ====================
impl_api_method!(GetUsersWithTopGameCounts, db_get_users_with_top_game_counts);
impl_api_method!(GetUserProfile, db_get_user_profile);

pub const INVENTORY_FUNCTIONS_IMPL: [ApiMethodImpl; 14] = [
    /*                         get_replay_match_list2           */
    /* ======================================================== */
    api_method_impl!(GetReplayMatchList),
    api_method_impl!(GetReplayMatchDetail),
    api_method_impl!(GetGameStateRowsForMatch),
    api_method_impl!(GetLastGameStatesForMatch),
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
    /*                         user_friends           */
    /* ======================================================== */
    api_method_impl!(UserAddFriend),
    api_method_impl!(UserDeleteFriend),
    api_method_impl!(UserGetFriends),
    /*                         user_profiles           */
    /* ======================================================== */
    api_method_impl!(GetUsersWithTopGameCounts),
    api_method_impl!(GetUserProfile),
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

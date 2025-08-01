use dioxus::{html::elements, prelude::*};
use game::tet::GameState;

use crate::{
    comp::{bot_player::BotPlayer, game_display::GameDisplay},
    network::NetworkState,
};

/// Home page
#[component]
pub fn Home() -> Element {
    let game_state = use_signal(GameState::new_random);
    let own_id = use_context::<NetworkState>().current_node_id;
    let Some(own_id) = own_id.read().clone() else {
        return rsx! {"loading..."};
    };

    rsx! {
        BotPlayer {game_state}
        article { style: "height: 80dvh; display: flex;",
            // style: "display: flex;",
            GameDisplay { game_state, node_id: own_id }
        }
    }
}

use dioxus::prelude::*;
use game::tet::GameState;

use crate::comp::{
    bot_player::BotPlayer, chat::chat_window_mini::MiniChatOverlay,
    game_display::GameDisplay,
};

/// Home page
#[component]
pub fn Home() -> Element {
    let game_state = use_signal(GameState::empty);

    rsx! {
        BotPlayer {game_state}
        article { style: "height: 80dvh; display: flex;",
            // style: "display: flex;",
            GameDisplay { game_state }
        }
        MiniChatOverlay {}
    }
}

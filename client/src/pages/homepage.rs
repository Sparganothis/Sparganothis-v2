use std::{collections::VecDeque, time::Duration};

use dioxus::prelude::*;
use dioxus_sdk::utils::timing::use_interval;
use game::{
    bot::{wordpress_blog_bot::WordpressBlogBot, TetBot},
    tet::{GameState, TetAction},
};

use crate::comp::{bot_player::BotPlayer, game_display::GameDisplay};

/// Home page
#[component]
pub fn Home() -> Element {
    let mut game_state = use_signal(GameState::empty);

    rsx! {
        BotPlayer {game_state}
        article { style: "height: 80dvh; display: flex;",
            // style: "display: flex;",
            GameDisplay { game_state }
        }
    }
}

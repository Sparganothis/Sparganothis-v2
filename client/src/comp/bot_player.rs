use std::collections::VecDeque;

use dioxus::prelude::*;
use dioxus_sdk::utils::timing::use_interval;
use game::{
    bot::{wordpress_blog_bot::WordpressBlogBot, TetBot},
    tet::{GameState, TetAction},
};

use crate::localstorage::use_game_settings;

#[component]
pub fn BotPlayer(game_state: Signal<GameState>) -> Element {
    let mut pending_actions = use_signal(VecDeque::<TetAction>::new);

    let settings = use_game_settings();
    let _interv = settings.game.auto_softdrop_interval;
    use_interval(_interv, move || {
        let mut g = game_state.write();
        let mut p = pending_actions.write();

        if g.game_over() {
            *g = GameState::new_random();
            return;
        }
        if p.is_empty() {
            if let Ok(r) = WordpressBlogBot.choose_move(&g) {
                *p = VecDeque::from_iter(r.into_iter());
            }
        }

        if let Some(a) = p.pop_front() {
            if let Ok(new_state) = g.try_action(a, 0) {
                *g = new_state;
            }
        }
    });
    rsx! {
        "bot"
    }
}

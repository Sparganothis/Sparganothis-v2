use std::{collections::VecDeque, time::Duration};

use dioxus::prelude::*;
use dioxus_sdk::utils::timing::use_interval;
use game::{
    bot::{wordpress_blog_bot::WordpressBlogBot, TetBot},
    tet::{GameState, TetAction},
};

#[component]
pub fn BotPlayer(game_state: Signal<GameState>) -> Element {
    let mut pending_actions = use_signal(VecDeque::<TetAction>::new);

    use_interval(Duration::from_secs_f32(0.1), move || {
        let mut g = game_state.write();
        let mut p = pending_actions.write();

        if g.game_over() {
            *g = GameState::empty();
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

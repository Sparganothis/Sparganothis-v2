use std::{collections::VecDeque, time::Duration};

use crate::comp::game_display::*;
use crate::route::Route;
use dioxus::prelude::*;
use dioxus_sdk::utils::timing::use_interval;
use game::{bot::{wordpress_blog_bot::WordpressBlogBot, TetBot}, tet::{GameState, TetAction}};
use dioxus::logger::tracing::info;
/// Home page
#[component]
pub fn Home() -> Element {
    let mut game_state = use_signal(GameState::empty);
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
        article { style: "height: 80dvh; display: flex;",
            // style: "display: flex;",
            GameDisplay { game_state }
        
        }
    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div { id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p {
                "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components."
            }

            // Navigation links
            Link { to: Route::Blog { id: id - 1 }, "Previous" }
            span { " <---> " }
            Link { to: Route::Blog { id: id + 1 }, "Next" }
        }
    }
}

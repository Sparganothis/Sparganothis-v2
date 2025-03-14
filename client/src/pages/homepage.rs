use std::time::Duration;

use crate::comp::game_display::*;
use crate::route::Route;
use dioxus::prelude::*;
use dioxus_sdk::utils::timing::use_interval;
use game::tet::{GameState, TetAction};
use dioxus::logger::tracing::info;
/// Home page
#[component]
pub fn Home() -> Element {
    let mut game_state = use_signal(GameState::empty);
    use_interval(Duration::from_secs_f32(0.1), move || {
        let mut g = game_state.write();
        
        if g.game_over() {
            *g = GameState::empty();
        } else {
            for _ in 0..10 {
                if let Ok(new_state) = g.try_action(TetAction::random(), 0) {
                    *g = new_state;
                    break;
                }
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

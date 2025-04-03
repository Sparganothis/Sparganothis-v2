use dioxus::prelude::*;
use game::tet::GameState;

use crate::comp::{
    bot_player::BotPlayer, game_display::GameDisplay,
    input::GameInputCaptureParent,
};

/// Home page
#[component]
pub fn Singleplayer() -> Element {
    let game_state = use_signal(GameState::empty);
    let mut game_event = use_signal(|| None);
    let on_user_event =
        Callback::new(move |event: game::input::events::GameInputEvent| {
            game_event.set(Some(event.clone()));
            tracing::info!("GameInputEvent, {:#?}", event);
        });

    rsx! {
        GameInputCaptureParent {
            on_user_event,

            article { style: "height: 80dvh; display: flex;",
                GameDisplay { game_state }
            }

            "GAME EVENT: {*game_event.read():#?}"
        }
    }
}

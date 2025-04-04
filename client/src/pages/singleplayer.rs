use dioxus::prelude::*;
use game::{input::{events::GameInputEvent, input_manager::GameInputManager}, tet::{GameState, TetAction}};

use crate::comp::{game_display::GameDisplay, input::GameInputCaptureParent};

/// Home page
#[component]
pub fn Singleplayer() -> Element {
    let mut game_state_w = use_signal(GameState::empty);
    let mut input_manager = use_signal(|| GameInputManager::new());
    let game_state = use_memo(move || {game_state_w.read().clone()});
    // let mut game_event = use_signal(|| None);
    let on_tet_action = Callback::new(move |action: TetAction| {
        if let Ok(next_state) = game_state.read().try_action(action, 0) {
            game_state_w.set(next_state);
        }
    });
    
    let on_user_event =
        Callback::new(move |event: GameInputEvent| {
            if let Some(action) = input_manager.write().on_user_event(event) {
                on_tet_action.call(action);
            }
        });
    rsx! {
        GameInputCaptureParent {
            on_user_event,

            article { style: "height: 80dvh; display: flex;",
                GameDisplay { game_state }
            }
        }
    }
}

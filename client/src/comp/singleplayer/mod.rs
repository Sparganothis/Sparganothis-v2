use dioxus::prelude::*;
use std::sync::Arc;

use futures_util::pin_mut;
use game::{
    input::{callback_manager::CallbackManager, events::GameInputEvent},
    tet::{GameState, TetAction},
    timestamp::get_timestamp_now_ms,
};
use n0_future::StreamExt;
use tokio::sync::RwLock;
use tracing::warn;

use crate::{
    comp::{game_display::GameDisplay, input::GameInputCaptureParent},
    localstorage::use_game_settings,
};

/// Basic single player implementation with default rules.
#[component]
pub fn SingleplayerGameBoardBasic() -> Element {
    let mut game_state = use_signal(move || GameState::empty());
    let set_next_state = use_callback(move |c: GameState| {
        if c.game_over() {
            game_state.set(GameState::empty());
        } else {
            game_state.set(c);
        }
    });

    rsx! {
        GameBoardInputAndDisplay {game_state, set_next_state: set_next_state}
    }
}


/// User Game board without fullscreen parent
#[component]
pub fn GameBoardInputAndDisplay(game_state: ReadOnlySignal<GameState>, 
    set_next_state: Callback<GameState>,) -> Element {
    // let mut game_state_w = use_signal(GameState::empty);
    // let game_state = use_memo(move || game_state_w.read().clone());

    let on_tet_action = Callback::new(move |action: TetAction| {
        let old_state = game_state.read().clone();
        if old_state.game_over() {
            // set_next_state.call(GameState::empty());
            return;
        }
        if let Ok(next_state) =
            old_state.try_action(action, get_timestamp_now_ms())
        {
            set_next_state.call(next_state);
        }
    });
    let ticket_manager = use_coroutine(
        move |mut _r: UnboundedReceiver<(GameState, GameInputEvent)>| async move {
            let callback_manager = CallbackManager::new2();
            let mut s = use_game_settings();
            let arc_s = Arc::new(RwLock::new(s));
            let _s = callback_manager.main_loop(_r, arc_s.clone()).await;
            pin_mut!(_s);
            while let Some(action) = _s.next().await {
                on_tet_action.call(action);
                let s2 = use_game_settings();
                if s2 != s {
                    s = s2;
                    *arc_s.write().await = s;
                }
            }
            warn!("ZZZ ====???+++++????+?+?+?+?+ EVENT STRREAM FINISH");
        },
    );

    let on_user_event = Callback::new(move |event: GameInputEvent| {
        let old_state = game_state.read().clone();
        ticket_manager.send((old_state, event));
    });
    rsx! {
        GameInputCaptureParent {
            on_user_event,

            GameDisplay { game_state }
        }
    }
}

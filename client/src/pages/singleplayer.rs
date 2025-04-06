use std::sync::Arc;

use dioxus::prelude::*;
use futures_util::pin_mut;
use game::{
    input::{callback_manager::CallbackManager, events::GameInputEvent},
    tet::{GameState, TetAction},
    timestamp::get_timestamp_now_ms,
};
use n0_future::StreamExt;
use tokio::sync::Mutex;
use tracing::warn;

use crate::{
    comp::{game_display::GameDisplay, input::GameInputCaptureParent},
    localstorage::use_game_settings,
};

/// Home page
#[component]
pub fn Singleplayer() -> Element {
    rsx! {
        article {
            style: "height: 80dvh; display: flex;",
            SingleplayerGameBoard {}
        }
    }
}

/// User Game board without fullscreen parent
#[component]
pub fn SingleplayerGameBoard() -> Element {
    let mut game_state_w = use_signal(GameState::empty);
    let game_state = use_memo(move || game_state_w.read().clone());

    let on_tet_action = Callback::new(move |action: TetAction| {
        let old_state = game_state.read().clone();
        if old_state.game_over() {
            game_state_w.set(GameState::empty());
            return;
        }
        if let Ok(next_state) =
            old_state.try_action(action, get_timestamp_now_ms())
        {
            game_state_w.set(next_state);
        }
    });
    let ticket_manager = use_coroutine(
        move |mut _r: UnboundedReceiver<GameInputEvent>| async move {
            let callback_manager = CallbackManager::new2();
            let mut s = use_game_settings();
            let arc_s = Arc::new(Mutex::new(s));
            let _s = callback_manager.main_loop(_r, arc_s.clone()).await;
            pin_mut!(_s);
            while let Some(action) = _s.next().await {
                on_tet_action.call(action);
                let s2 = use_game_settings();
                if s2 != s {
                    s = s2;
                    *arc_s.lock().await = s;
                }
            }
            warn!("ZZZ ====???+++++????+?+?+?+?+ EVENT STRREAM FINISH");
        },
    );

    let on_user_event = Callback::new(move |event: GameInputEvent| {
        ticket_manager.send(event);
    });
    rsx! {
        GameInputCaptureParent {
            on_user_event,

            GameDisplay { game_state }
        }
    }
}

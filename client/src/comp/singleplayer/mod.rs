use dioxus::prelude::*;
use std::sync::Arc;

use futures_util::pin_mut;
use game::{
    input::{
        callback_manager::InputCallbackManagerRule, events::GameInputEvent,
    },
    state_manager::GameStateManager,
    tet::{get_random_seed, GameState, TetAction},
    timestamp::get_timestamp_now_ms,
};
use n0_future::{task::AbortOnDropHandle, StreamExt};
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
        game_state.set(c);
    });

    rsx! {
        GameBoardInputAndDisplay {game_state, set_next_state: set_next_state}
    }
}

/// User Game board without fullscreen parent
#[component]
pub fn GameBoardInputAndDisplay(
    game_state: ReadOnlySignal<GameState>,
    set_next_state: Callback<GameState>,
) -> Element {
    let ticket_manager = use_coroutine(
        move |mut _r: UnboundedReceiver<GameInputEvent>| async move {
            let mut game_state_manager = GameStateManager::new(
                &get_random_seed(),
                get_timestamp_now_ms(),
            );
            let mut s = use_game_settings();
            let arc_s = Arc::new(RwLock::new(s));
            let callback_manager = InputCallbackManagerRule::new(
                _r,
                game_state_manager.read_state_stream(),
                arc_s.clone(),
            );
            game_state_manager.add_rule(Arc::new(callback_manager));
            let g2 = game_state_manager.clone();
            let stream = game_state_manager.read_state_stream();
            let main_loop =
                AbortOnDropHandle::new(n0_future::task::spawn(async move {
                    g2.main_loop().await
                }));
            pin_mut!(stream);
            while let Some(next_state) = stream.next().await {
                set_next_state.call(next_state);
                let s2 = use_game_settings();
                if s2 != s {
                    s = s2;
                    *arc_s.write().await = s;
                }
            }
            warn!("ZZZ ====???+++++????+?+?+?+?+ EVENT STRREAM FINISH");
            drop(main_loop);
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

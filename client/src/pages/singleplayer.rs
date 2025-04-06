use dioxus::prelude::*;
use dioxus_sdk::utils::timing::use_debounce;
use game::{
    input::{
        callback_manager::CallbackManager,
        events::GameInputEvent,
        input_manager::{GameInputManager, UserEvent},
    },
    tet::{GameState, TetAction},
    timestamp::get_timestamp_now_ms,
};
use n0_future::StreamExt;
use tracing::warn;

use crate::{comp::{game_display::GameDisplay, input::GameInputCaptureParent}, localstorage::use_game_settings};

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
    let mut input_manager = use_signal(|| GameInputManager::new());
    let game_state = use_memo(move || game_state_w.read().clone());
    // let mut game_event = use_signal(|| None);

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
    let ticket_manager =
        use_coroutine(move |mut _r: UnboundedReceiver<UserEvent>| async move {
            use futures_util::FutureExt;
            let callback_manager = CallbackManager::new2();
            let mut zzz = 0;
            loop {
                zzz += 1;
                let game_settings = use_game_settings();
                let (duration_ms, items) =
                    callback_manager.get_sleep_duration_ms().await;
                for _move in items {
                    let x = input_manager.write().callback_after_wait(_move, game_settings);
                    let y = callback_manager.accept_user_event(x).await;
                    if let Some(action) = y {
                        on_tet_action.call(action);
                    }
                }
                let duration =
                    std::time::Duration::from_millis(duration_ms as u64);

                tokio::select! {
                    event = _r.next().fuse() => {
                        let Some(event) = event else {
                            tracing::warn!("ticket manger loop end: coro end");
                            break;
                        };
                        let y = callback_manager.accept_user_event(event).await;
                        if let Some(action) = y {
                            on_tet_action.call(action);
                        }
                        continue;
                    }
                    _not = callback_manager.notified().fuse() => {
                        continue;
                    }

                    _sl = n0_future::time::sleep(duration).fuse() => {
                        continue;
                    }
                }
            }
            warn!("ZZZ {zzz} ====???+++++????+?+?+?+?+ EVENT STRREAM FINISH");
        });

    let on_user_event = Callback::new(move |event: GameInputEvent| {
        let settings = use_game_settings();
        let event = input_manager.write().on_user_keyboard_event(event, settings);
        ticket_manager.send(event);
    });
    rsx! {
        GameInputCaptureParent {
            on_user_event,

                GameDisplay { game_state }
            
        }
    }
}

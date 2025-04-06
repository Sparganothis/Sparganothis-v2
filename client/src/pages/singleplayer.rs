use dioxus::prelude::*;
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

use crate::comp::{game_display::GameDisplay, input::GameInputCaptureParent};

/// Home page
#[component]
pub fn Singleplayer() -> Element {
    let mut game_state_w = use_signal(GameState::empty);
    let mut input_manager = use_signal(|| GameInputManager::new());
    let game_state = use_memo(move || game_state_w.read().clone());
    // let mut game_event = use_signal(|| None);
    let on_tet_action = Callback::new(move |action: TetAction| {
        warn!("ZZZ ===> TETACTION {:?}", action);
        if let Ok(next_state) =
            game_state.read().try_action(action, get_timestamp_now_ms())
        {
            game_state_w.set(next_state);
        }
    });
    let ticket_manager =
        use_coroutine(move |mut _r: UnboundedReceiver<UserEvent>| async move {
            use futures_util::FutureExt;
            let callback_manager = CallbackManager::new();
            let mut zzz = 0;
            loop {
                zzz += 1;
                warn!("\n\n ZZZ {zzz} ============== INIT ===============");
                let (duration_ms, items) =
                    callback_manager.get_sleep_duration_ms().await;
                warn!("INIT {} items", items.len());
                for _move in items {
                    let x = input_manager.write().callback_after_wait(_move);
                    let y = callback_manager.accept_user_event(x).await;
                    if let Some(action) = y {
                        on_tet_action.call(action);
                    }
                }
                let duration =
                    std::time::Duration::from_millis(duration_ms as u64);

                warn!("\n ZZZ {zzz} ============== SELECT {duration:?} ===============");
                tokio::select! {
                    event = _r.next().fuse() => {
                        warn!("ZZZ event! {:#?}", event);
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
                        warn!("ZZZ {zzz} notified!!!");
                        continue;
                    }

                    _sl = n0_future::time::sleep(duration).fuse() => {
                        warn!("ZZZ {zzz} slept for {:?}!!!", duration);
                        continue;
                    }
                }
            }
            warn!("ZZZ {zzz} ====???+++++????+?+?+?+?+ EVENT STRREAM FINISH");
        });

    let on_user_event = Callback::new(move |event: GameInputEvent| {
        let event = input_manager.write().on_user_keyboard_event(event);
        ticket_manager.send(event);
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

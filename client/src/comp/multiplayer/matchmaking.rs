use std::time::Duration;

use dioxus::prelude::*;
use futures_util::FutureExt;
use game::api::game_match::{GameMatch, GameMatchType};
use n0_future::StreamExt;
use protocol::{
    game_matchmaker::find_game,
    user_identity::NodeIdentity,
};
use tracing::{info, warn};

use crate::network::{GlobalChatClientContext, NetworkState};

#[component]
pub fn MatchmakingWindow(
    user_match_type: ReadOnlySignal<GameMatchType>,
    on_opponent_confirm: Callback<GameMatch<NodeIdentity>>,
    on_matchmaking_failed: Callback<String>,
) -> Element {
    let chat = use_context::<GlobalChatClientContext>().chat;
    let mut clicked = use_signal(|| false);

    let mut timer_w = use_signal(move || 0);
    let timer = use_memo(move || timer_w.read().clone());
    let attempt_timeout_secs = 10;
    let attempts = 5;
    let coro = use_coroutine(move |mut _r| async move {
        while let Some(_x) = _r.next().await {
            let Some(global_chat) = chat.chat.peek().clone() else {
                warn!("no chat!");
                continue;
            };

            info!("\n\n\n >>>>> > START MATCHMAKING \n\n");

            let timeout = std::time::Duration::from_secs(attempt_timeout_secs);
            let mut game_fut =
                find_game(user_match_type.peek().clone(), global_chat, timeout, attempts).fuse().boxed();
            timer_w.set(0);
            let game = loop {
                tokio::select! {
                    _ = n0_future::time::sleep(Duration::from_secs(1)).fuse() => {
                        let old = timer.peek().clone() + 1;
                        if old > (attempt_timeout_secs + 1) * attempts as u64 {
                            break Err("coro timeout".to_string());
                        }
                        timer_w.set(old);

                        continue;
                    }
                    game = &mut game_fut => {
                        break game.map_err(|e| e.to_string());
                    }
                }
            };
            match game {
                Ok(from) => {
                    on_opponent_confirm.call(from);
                }
                Err(e) => {
                    let txt = format!("err: {e}");
                    on_matchmaking_failed.call(txt.clone());
                }
            }
            
            info!("\n\n\n >>>>> > END MATCHMAKING \n\n");
        }
    });

    let mm = use_context::<NetworkState>().is_connected;

    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                height: 4.5rem;
                width: 15rem;
            ",
            if *mm.read() {
                if !*clicked.read() {
                    button {
                        onclick: move |_| {
                            coro.send(());
                            clicked.set(true);
                        },
                        "Look for game!"
                    }
                } else {
                    h3 {
                        "Looking {timer}/{attempt_timeout_secs}..."
                    }
                }
            } else {
                h3 {
                    "Connecting..."
                }
            }
        }
    }
}

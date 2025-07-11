use std::time::Duration;

use dioxus::prelude::*;
use futures_util::FutureExt;
use game::{
    api::game_match::{GameMatch, GameMatchType},
    timestamp::get_timestamp_now_ms,
};
use n0_future::StreamExt;
use protocol::{
    global_matchmaker::GlobalMatchmaker,
    server_chat_api::{
        api_declarations::{
            RunMultiplayerMatchmakerPhase1, RunMultiplayerMatchmakerPhase2,
            SendNewMatch,
        },
        client_api_manager::ClientApiManager,
    },
    user_identity::NodeIdentity,
};
use tracing::{info, warn};

use crate::network::{GlobalChatClientContext, NetworkState};

async fn run_matchmaker_once(
    api: ClientApiManager,
    game_type: GameMatchType,
) -> anyhow::Result<GameMatch<NodeIdentity>> {
    let phase1 = api
        .call_method::<RunMultiplayerMatchmakerPhase1>(game_type.clone())
        .await?;
    let phase2 = api
        .call_method::<RunMultiplayerMatchmakerPhase2>((game_type, phase1))
        .await?;
    Ok(phase2)
}

async fn run_matchmaker(
    mm: GlobalMatchmaker,
    api: ClientApiManager,
    game_type: GameMatchType,
) -> anyhow::Result<GameMatch<NodeIdentity>> {
    let max_time = get_timestamp_now_ms() + 29000;
    let mut xx = 0;
    while get_timestamp_now_ms() < max_time {
        let r = run_matchmaker_once(api.clone(), game_type.clone()).await;
        xx += 1;
        if let Ok(r) = r {
            return Ok(r);
        };
        mm.sleep(Duration::from_millis(500)).await;
    }
    anyhow::bail!("Matchmaker timed out!")
}

#[component]
pub fn MatchmakingWindow(
    mm: GlobalMatchmaker,
    api: ClientApiManager,
    user_match_type: ReadOnlySignal<GameMatchType>,
    on_opponent_confirm: Callback<GameMatch<NodeIdentity>>,
    on_matchmaking_failed: Callback<String>,
) -> Element {
    let mut clicked = use_signal(|| false);
    let mut timer_w = use_signal(move || 0);
    let timer = use_memo(move || timer_w.read().clone());

    let coro = use_coroutine(move |mut _r| {
        let api = api.clone();
        let mm = mm.clone();

        async move {
            while let Some(_x) = _r.next().await {
                let mut game_fut = run_matchmaker(
                    mm.clone(),
                    api.clone(),
                    user_match_type.read().clone(),
                )
                .fuse()
                .boxed();
                timer_w.set(0);
                let game = loop {
                    tokio::select! {
                        _ = n0_future::time::sleep(Duration::from_secs(1)).fuse() => {
                            let old = timer.peek().clone() + 1;
                            timer_w.set(old);

                            continue;
                        }
                        game = &mut game_fut => {
                            break game.map_err(|e| format!("MatchmakingWindow: {e:#?}"));
                        }
                    }
                };
                match game {
                    Ok(from) => {
                        tracing::info!("confirm matchmaking!");
                        let from2 = from.clone();
                        send_new_match(from2).await;
                        on_opponent_confirm.call(from);
                    }
                    Err(e) => {
                        let txt = format!("matchmaking error: {e}");
                        on_matchmaking_failed.call(txt.clone());
                    }
                }

                info!("\n\n\n >>>>> > END MATCHMAKING \n\n");
            }
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
                        "Looking for {user_match_type:?}: {timer}s..."
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

async fn send_new_match(m: GameMatch<NodeIdentity>) {
    let api = use_context::<NetworkState>()
        .client_api_manager
        .peek()
        .clone();
    tracing::info!("send_new_match_coro()");
    let Some(api) = api else {
        warn!("no api! skipping send_new_match m,essage...");
        return;
    };
    if let Err(e) = api.call_method::<SendNewMatch>((m,)).await {
        warn!("FAILED TO SEND SendNewMatch method to backend! {e:#?}");
        return;
    }
}

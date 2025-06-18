use std::sync::Arc;

use async_channel::unbounded;
use dioxus::prelude::*;
use futures_util::pin_mut;
use game::{
    api::game_match::GameMatch, futures_channel, input::events::GameInputEvent,
    state_manager::GameStateManager, tet::GameState,
};
use game_net::{
    get_1v1_player_state_manager, get_spectator_state_manager,
    Game1v1MatchChatController,
};
use n0_future::{task::AbortOnDropHandle, StreamExt};
use protocol::{
    global_matchmaker::GlobalMatchmaker, user_identity::NodeIdentity,
};
use tokio::sync::RwLock;

use crate::{
    comp::{
        game_display::GameDisplay, input::GameInputCaptureParent,
        singleplayer::SingleplayerGameBoardBasic,
    },
    localstorage::use_game_settings,
};

#[component]
pub fn Play1v1WindowTitle(
    mm: GlobalMatchmaker,
    game_match: GameMatch<NodeIdentity>,
) -> Element {
    let our_node = mm.own_node_identity();

    rsx! {
        div {
            "play 1v1: "
            TitleUsernameSpan { node: game_match.users[0], is_current_user: our_node == game_match.users[0]}
            " vs. "
            TitleUsernameSpan { node: game_match.users[1], is_current_user: our_node == game_match.users[1]}
        }
    }
}

#[component]
pub fn Spectate1v1WindowTitle(
    mm: GlobalMatchmaker,
    game_match: GameMatch<NodeIdentity>,
) -> Element {
    let our_node = mm.own_node_identity();

    rsx! {
        div {
            "spectate 1v1: "
            TitleUsernameSpan { node: game_match.users[0], is_current_user: our_node == game_match.users[0]}
            " vs. "
            TitleUsernameSpan { node: game_match.users[1], is_current_user: our_node == game_match.users[1]}
        }
    }
}

#[component]
fn TitleUsernameSpan(node: NodeIdentity, is_current_user: bool) -> Element {
    rsx! {
        span {
            style: "color: {node.html_color()}",
            "{node.nickname()}"
        }
        if is_current_user {
            span {
                style: "color: gray; font-size: 1rem;",
                " (you) "
            }
        }
    }
}

#[component]
pub fn Play1v1FullscreenWindow(
    mm: GlobalMatchmaker,
    game_match: GameMatch<NodeIdentity>,
) -> Element {
    // let opponent_idx = if game_match.users[0] == mm.own_node_identity() {
    //     1
    // } else { 0};

    let match_chat = use_resource(move || {
        let mm2 = mm.clone();
        let game_match = game_match.clone();

        async move {
            let match_chat = game_net::join_1v1_match(mm2, game_match).await;
            match_chat.ok()
        }
    });

    let match_chat = use_memo(move || match_chat.read().clone().flatten());

    rsx! {
        div {
            style: "display: flex; flex-direction: row; container-type: size; width: 100%; height: 100%;",
            if let Some(cc) = match_chat.read().clone() {
                Play1v1FullScreenWindowInner {cc}
            } else {
                h1 {
                    "Chat loading .... "
                }
            }
        }
    }
}

#[component]
fn Play1v1FullScreenWindowInner(cc: Game1v1MatchChatController) -> Element {
    tracing::info!("Play1v1FullScreenWindowInner()");
    let (input_tx, input_rx) = futures_channel::mpsc::unbounded();
    let settings = Arc::new(RwLock::new(use_game_settings()));
    let play_state_manager =
        get_1v1_player_state_manager(cc.clone(), settings, input_rx);
    let spectator_manager = get_spectator_state_manager(cc);
    let on_user_event = Callback::new(move |event: GameInputEvent| {
        tracing::info!(
            "Play1v1FullScreenWindowInner(): on user event: {:#?}",
            event
        );
        let _ = input_tx.unbounded_send(event);
    });

    rsx! {
        div {
            style: "width: 50cqw; height: 100cqh",
            GameInputCaptureParent {
                on_user_event,

                GameStateManagerDisplay {manager: play_state_manager}
            }
        }
        div {
            style: "width: 50cqw; height: 100cqh;",
            GameStateManagerDisplay {manager: spectator_manager}
        }
    }
}

#[component]
pub fn Spectate1v1FullScreenWindow(
    mm: GlobalMatchmaker,
    game_match: GameMatch<NodeIdentity>,
) -> Element {
    rsx! {
        h1 {
            "TODO : SPECTATE plz wait for impl"
        }
    }
}

#[component]
fn GameStateManagerDisplay(manager: GameStateManager) -> Element {
    let mut game_state = use_signal(GameState::empty);

    let _coro = use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let m2 = manager.clone();

        let main_loop = AbortOnDropHandle::new(n0_future::task::spawn(async move {
            m2.main_loop().await
        }));
        let m2 = manager.clone();

        async move {
            let stream = m2.read_state_stream();
            pin_mut!(stream);
            while let Some(s) = stream.next().await {
                game_state.set(s);
            }
            let main_loop_result = main_loop.await;
            tracing::info!("GameStateManagerDisplay main loop finalized: {:#?}", main_loop_result);
        }
    });

    rsx! {
        GameDisplay {game_state }
    }
}

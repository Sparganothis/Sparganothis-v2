use dioxus::prelude::*;
use game::api::game_match::GameMatch;
use protocol::{
    global_matchmaker::GlobalMatchmaker, user_identity::NodeIdentity,
};

use crate::comp::singleplayer::SingleplayerGameBoardBasic;

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
    rsx! {
        div {
            style: "display: flex; flex-direction: row; container-type: size; width: 100%; height: 100%;",
            div {
                style: "width: 50cqw; height: 100cqh",
                SingleplayerGameBoardBasic {}
            }
            div {
                style: "width: 50cqw; height: 100cqh;",
                SingleplayerGameBoardBasic {}

            }
        }
    }
}

#[component]
pub fn Spectate1v1FullScreenWindow(
    mm: GlobalMatchmaker,
    game_match: GameMatch<NodeIdentity>,
) -> Element {
    rsx! {
        Play1v1FullscreenWindow {
            mm, game_match
        }
    }
}

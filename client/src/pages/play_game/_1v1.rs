use dioxus::prelude::*;
use game::api::game_match::GameMatch;
use protocol::user_identity::NodeIdentity;

use crate::{
    comp::multiplayer::_1v1::{
        Play1v1FullscreenWindow, Play1v1WindowTitle,
        Spectate1v1FullScreenWindow, Spectate1v1WindowTitle,
    },
    network::NetworkState,
    route::UrlParam,
};

#[component]
pub fn Play1v1Page(
    game_match: ReadOnlySignal<UrlParam<GameMatch<NodeIdentity>>>,
) -> Element {
    let NetworkState {
        global_mm,
        ..
    } = use_context::<NetworkState>();

    let mm = global_mm.read().clone();
    let Some(mm) = mm else {
        return rsx! {
            h1 {
                "Connecting..."
            }
        };
    };

    let game_match = use_memo(move || game_match.read().0.clone());

    let mm1 = mm.clone();
    let current_node_id_participates = use_memo(move || {
        let m = game_match.read();
        let users = &m.users;
        let our_id = mm.own_node_identity();
        users.contains(&our_id)
    });

    let mm = mm1.clone();
    rsx! {
        article {
            style: "
                height: 100%;
                width: 100%;
                display: flex;
                flex-direction: column;
            ",
            header {
                h1 {

                    if *current_node_id_participates.read() {
                        Play1v1WindowTitle {
                            mm: mm1,
                            game_match: game_match.read().clone()
                        }
                    } else {
                        Spectate1v1WindowTitle {
                            mm: mm1,
                            game_match: game_match.read().clone()
                        }
                    }
                }
            }

            div {
                style: "width: 100%; height: 100%; flex-grow: 1;",

                if *current_node_id_participates.read() {
                    Play1v1FullscreenWindow {
                        mm: mm,
                        game_match: game_match.read().clone()
                    }
                } else {
                    Spectate1v1FullScreenWindow {
                        mm: mm,
                        game_match: game_match.read().clone()
                    }
                }
            }

            footer {
                h4 {
                    "Match {game_match.read().match_id}",
                }
            }
        }
    }
}

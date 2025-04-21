use dioxus::prelude::*;
use game::api::game_match::GameMatch;
use protocol::user_identity::NodeIdentity;

use crate::{comp::multiplayer::_1v1::Play1v1FullscreenWindow, network::NetworkState, route::UrlParam};

#[component]
pub fn Play1v1Page(game_match: ReadOnlySignal<UrlParam<GameMatch<NodeIdentity>>>) -> Element {
    let NetworkState {
        global_mm,
        reset_network,
        global_mm_loading,
        bootstrap_idx,
        ..
    } = use_context::<NetworkState>();

    let game_match = use_memo(move ||  game_match.read().0.clone());

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
                    "1v1 vs. yourself (lol)"
                }
            }

            Play1v1FullscreenWindow {
                game_match: game_match.read().clone()
            }

            footer {
                h4 {
                    "Match {game_match.read().match_id}",
                }
            }
        }
    }
}

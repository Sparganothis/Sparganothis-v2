use dioxus::prelude::*;
use game::api::game_match::GameMatch;
use protocol::user_identity::NodeIdentity;

use crate::{comp::multiplayer::_1v1::Outcome1v1WindowTitle, network::NetworkState, route::UrlParam};

#[component]
pub fn Game1v1OutcomePage(
    game_match: ReadOnlySignal<UrlParam<GameMatch<NodeIdentity>>>,
) -> Element {
    let game_match = use_memo(move || game_match.read().0.clone());
    let Some(mm) = use_context::<NetworkState>().global_mm.read().clone() else {
        return rsx!{
            "loading..."
        }
    };
    

    rsx! {
        div {

            style: "display:flex; flex-direction:column;",

            article {
                style: "height: 10cqh;",

                h1 {
                    Outcome1v1WindowTitle { game_match:  game_match.read().clone(), mm: mm.clone()}
                }
            }

            article {
                style: "display: flex; flex-direction: row;",
                div {
                    style:"width:50cqw; height:80cqh",
                    "TODO1"
                }
                div {
                    style:"width:50cqw; height:80cqh",
                    "TODO2"
                }
            }
        }
    }
}

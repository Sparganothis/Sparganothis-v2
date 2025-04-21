use dioxus::prelude::*;

use crate::{comp::multiplayer::matchmaking::MatchmakingWindow, route::{Route, UrlParam}};

#[component]
pub fn MatchmakingPage() -> Element {
    let mut item = use_signal(move || None);
    let mut error = use_signal(move || None);
    rsx! {
        article {
            style: "
                height: 100%;
                width: 100%;
            ",

            h1 {
                "Matchmaking {item.read().is_some()}"
            }

            if item.read().is_none() && error.read().is_none() {
                MatchmakingWindow {
                    user_match_type: game::api::game_match::GameMatchType::_1v1,
                    on_opponent_confirm: move |other: game::api::game_match::GameMatch<protocol::user_identity::NodeIdentity>| {
                        item.set(Some(other.clone()));
                        error.set(None);
                        navigator().push(Route::Play1v1Page{game_match: UrlParam(other)});
                    },
                    on_matchmaking_failed: move |e| {
                        println!("matchmaking failed: {e}");
                        error.set(Some(e));
                        item.set(None);
                    },
                }
            }
            button {
                onclick: move |_| {
                    item.set(None);
                    error.set(None);
                },
                class: "secondary",
                "Reset"
            }
            pre {
                "Item: {item:#?}"
            }
            if let Some(e) = error.read().as_ref() {
                h1 {
                    style: "color: red;",
                    "Error: {e}"
                }
            }
        }
    }
}

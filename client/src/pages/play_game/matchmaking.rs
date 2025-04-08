use dioxus::prelude::*;

use crate::comp::multiplayer::matchmaking::MatchmakingWindow;

#[component]
pub fn MatchmakingPage() -> Element {
    let mut item = use_signal(move || None);
    rsx! {
        article {
            style: "
                height: 100%;
                width: 100%;
            ",

            h1 {
                "Matchmaking {item.read().is_some()}"
            }

            MatchmakingWindow {
                user_match_type: game::api::game_match::GameMatchType::_1v1,
                on_opponent_confirm: move |other| {
                    item.set(Some(other));
                },
            }
            button {
                onclick: move |_| {
                    item.set(None);
                },
                class: "secondary",
                "Reset"
            }
            h1 {
                "Item: {item:#?}"
            }
        }
    }
}

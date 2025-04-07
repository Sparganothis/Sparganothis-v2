use dioxus::prelude::*;

use crate::comp::multiplayer::matchmaking::MatchmakingWindow;

#[component]
pub fn MatchmakingPage() -> Element {
    rsx! {
        article {
            style: "
                height: 100%;
                width: 100%;
            ",

            h1 {
                "Matchmaking"
            }
            MatchmakingWindow {

            }
        }
    }
}

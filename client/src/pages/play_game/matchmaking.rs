use dioxus::prelude::*;
use uuid::Uuid;

use crate::{
    comp::{cosmetic::Hline, multiplayer::matchmaking::MatchmakingWindow},
    network::NetworkState,
    route::{Route, UrlParam},
};

#[component]
pub fn MatchmakingPage() -> Element {
    let mut error = use_signal(move || None);
    let NetworkState {
        client_api_manager,
        global_mm,
        ..
    } = use_context::<NetworkState>();
    let (Some(api), Some(mm)) =
        (client_api_manager.read().clone(), global_mm.read().clone())
    else {
        return rsx! {
            "loading..."
        };
    };
    let own_node = mm.own_node_identity();

    rsx! {
        article {
            style: "
                height: 100%;
                width: 100%;
            ",

            h1 {
                "Matchmaking 1v1"
            }

            if error.read().is_none() {
                MatchmakingWindow {
                    mm,
                    api,
                    user_match_type: game::api::game_match::GameMatchType::_1v1,
                    on_opponent_confirm: move |other: game::api::game_match::GameMatch<protocol::user_identity::NodeIdentity>| {
                        navigator().push(Route::Play1v1Page{game_match: UrlParam(other)});
                    },
                    on_matchmaking_failed: move |e| {
                        println!("matchmaking failed: {e}");
                        error.set(Some(e));
                    },
                }
            }
            if let Some(e) = error.read().as_ref() {
                h1 {
                    style: "color: red;",
                    "Error: {e}"
                }

                button {
                    onclick: move |_| {
                        error.set(None);
                    },
                    class: "secondary",
                    "Reset"
                }
            }
            Hline {  }
            h1 {
                "Create Private 1v1 Room"
            }
            button {
                onclick: move |_| {
                    navigator().push(Route::Private1v1RoomLobbyPage { owner_id: UrlParam(own_node), room_uuid: Uuid::new_v4() });
                    
                },
                "Create private room!"
            }

        }
    }
}

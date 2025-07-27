use dioxus::prelude::*;
use game::api::game_match::GameMatch;
use protocol::{
    api::api_declarations::GetLastGameStatesForMatch,
    user_identity::NodeIdentity,
};

use crate::{
    comp::{
        game_display::GameDisplay, multiplayer::_1v1::Outcome1v1WindowTitle,
    },
    network::NetworkState,
    route::UrlParam,
};

#[component]
pub fn Game1v1OutcomePage(
    game_match: ReadOnlySignal<UrlParam<GameMatch<NodeIdentity>>>,
) -> Element {
    let game_match = use_memo(move || game_match.read().0.clone());
    let Some(mm) = use_context::<NetworkState>().global_mm.read().clone()
    else {
        return rsx! {
            "loading..."
        };
    };
    let Some(api) = use_context::<NetworkState>()
        .client_api_manager
        .read()
        .clone()
    else {
        return rsx! {
            "loading..."
        };
    };
    let mut error_str = use_signal(|| None);
    let last_moves = use_resource(move || {
        let game_match = game_match.read().clone();
        let api = api.clone();
        async move {
            let last_states = api
                .call_method::<GetLastGameStatesForMatch>(game_match)
                .await;
            match last_states {
                Err(e) => {
                    error_str.set(Some(format!("{e:#}")));
                    return vec![];
                }
                Ok(lst) => lst,
            }
        }
    });
    let last_moves =
        use_memo(move || last_moves.read().clone().unwrap_or(vec![]));
    let is_loaded = use_memo(move || last_moves.read().len() == 2);

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
                if let Some(e) = error_str.read().clone() {
                    pre {
                        style:"color:red",
                        "{e}"
                    }
                } else {
                    div {
                        style:"width:50cqw; height:80cqh",
                        if *is_loaded.read() {
                            GameDisplay {
                                game_state: last_moves.read()[0].clone(),
                            }
                        } else {
                            "loading..."
                        }
                    }
                    div {
                        style:"width:50cqw; height:80cqh",
                        if  *is_loaded.read() {
                            GameDisplay {
                                game_state: last_moves.read()[1].clone(),
                            }
                        } else {
                            "loading..."
                        }
                    }
                }
            }
        }
    }
}

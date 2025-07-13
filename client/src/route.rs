use std::fmt::Display;
use std::str::FromStr;

use base64::prelude::BASE64_URL_SAFE;
use base64::Engine;
use dioxus::prelude::*;
use game::api::game_match::GameMatch;
use iroh::NodeId;
use protocol::user_identity::{NodeIdentity, UserIdentity};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::app::GlobalUrlContext;
use crate::comp::chat::global_chat::GlobalMiniChatOverlayParent;
use crate::comp::nav::Nav;
use crate::pages::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(LayoutParent)]
    #[route("/")]
    Home {},

    #[nest("/my")]
        #[route("/my-profile")]
        MyProfilePage {},

        #[route("/my-main-settings")]
        MyMainSettings {},
        
        #[route("/my-button-settings")]
        MyButtonSettings {},
    #[end_nest]

    #[nest("/users")]
        #[route("/")]
        UsersRootDirectoryPage {},
        #[route("/user/:user_id")]
        UsersProfilePage {user_id: UrlParam<UserIdentity>},
    #[end_nest]

    #[nest("/chat")]
        #[route("/")]
        GlobalChatPage {},
    #[end_nest]

    #[route("/spectate-homepage/:node_id")]
    SpectateGamePage { node_id: NodeId },

    #[route("/i_am_a_robot_singleplayer")]
    IAmARobotSingleplayer {},

    #[nest("/play")]

        #[route("/")]
        PlayGameRootPage {},

        #[route("/singleplayer")]
        PlaySingleplayerPage {},

        #[route("/1v1/:game_match")]
        Play1v1Page {game_match: UrlParam<GameMatch<NodeIdentity>>},

        #[route("/matchmaking")]
        MatchmakingPage {},

        #[route("/replays")]
        ReplayHomePage {},

        #[route("/replays/1v1_match/:match_id")]
        Replay1v1Match {match_id: String},


        #[route("/private_lobby/:owner_id/:room_uuid")]
        PrivateLobbyPage{owner_id: UrlParam<NodeIdentity>, room_uuid: uuid::Uuid},
        
    #[end_nest]

    #[route("/:..x")]
    NotFoundPage { x: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlParam<T>(pub T);
impl<T: Serialize + for<'a> Deserialize<'a>> Display for UrlParam<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = bincode::serialize(&self.0).unwrap_or_default();
        let string = BASE64_URL_SAFE.encode(bytes);
        f.write_str(&string)?;
        Ok(())
    }
}

impl<T: Serialize + for<'a> Deserialize<'a>> FromStr for UrlParam<T> {
    type Err = bincode::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = BASE64_URL_SAFE.decode(s).unwrap_or_default();
        let value = bincode::deserialize(&bytes);
        value
    }
}

#[component]
fn NotFoundPage(x: Vec<String>) -> Element {
    let url = x.join("/");
    rsx! {
        h1 { "Not Found: /{url}" }
    }
}

#[component]
fn LayoutParent() -> Element {
    info!("\n\n     LayoutParent\n\n");
    let mut url_w = use_context::<GlobalUrlContext>().url_w;
    let mut route_w = use_context::<GlobalUrlContext>().route_w;
    use_effect(move || {
        let route = use_route::<Route>();
        url_w.set(route.to_string());
        route_w.set(route);
    });
    rsx! {
        NavbarLayout {}
    }
}

/// Shared navbar component.
#[component]
fn NavbarLayout() -> Element {
    info!("\n\n     NavbarLayout\n\n");
    rsx! {
        div {
            class: "container-fluid",
            style:  "
                height: 99%;
                display: flex; 
                flex-direction: column;
            ",

            div {
                style: "height: calc(max(7%, 80px));",
                Nav {}
            }
            main {
                style: "
                    overflow: auto;
                    height: calc(100% - (max(7%, 80px)));
                ",
                GlobalMiniChatOverlayParent {
                    Outlet::<Route> {}
                }
            }
        }
    }
}

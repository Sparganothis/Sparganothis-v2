use dioxus::prelude::*;
use iroh::NodeId;
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

    #[route("/my-profile")]
    MyProfilePage {},

    #[route("/my-main-settings")]
    MyMainSettings {},
    
    #[route("/my-button-settings")]
    MyButtonSettings {},

    #[route("/chat")]
    GlobalChatPage {},

    #[route("/spectate-homepage/:node_id")]
    SpectateGamePage { node_id: NodeId },

    #[route("/singleplayer")]
    Singleplayer {},

    #[route("/i_am_a_robot_singleplayer")]
    IAmARobotSingleplayer {},

    #[route("/:..x")]
    NotFound { x: Vec<String> },

}

#[component]
fn NotFound(x: Vec<String>) -> Element {
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

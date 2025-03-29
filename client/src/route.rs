use dioxus::prelude::*;
use iroh::NodeId;

use crate::app::GlobalUrlContext;
use crate::comp::nav::Nav;
use crate::pages::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(NavbarLayout)]
    #[route("/")]
    Home {},

    #[route("/my-profile")]
    MyProfilePage {},

    #[route("/chat")]
    GlobalChatPage {},

    #[route("/spectate-homepage/:node_id")]
    SpectateGamePage { node_id: NodeId },

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

/// Shared navbar component.
#[component]
fn NavbarLayout() -> Element {
    let mut url = use_context::<GlobalUrlContext>().url_w;
    use_effect(move || {
        let route = use_route::<Route>();
        url.set(route.to_string());
    });
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
                 Outlet::<Route> {} }
        }
    }
}

use dioxus::prelude::*;

use crate::comp::nav::Nav;
use crate::pages::*;
use crate::storage_demo::StorageDemo;

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

    #[route("/:..x")]
    NotFound { x: Vec<String> },

    #[route("/storage-demo")]
    StorageDemo {},
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

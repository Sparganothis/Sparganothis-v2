use dioxus::prelude::*;

use crate::constants::*;
use crate::pages::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(NavbarLayout)]
    #[route("/")]
    Home {},
    #[route("/chat/:id")]
    ChatPage { id: i8 },

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
fn Nav() -> Element {
    rsx! {
        nav {
            ul {
                li {
                    strong { "{APP_TITLE}" }
                }
            }
            ul {
                li {
                    Link { to: Route::Home {}, "Home" }
                }
                li {
                    Link { to: Route::ChatPage { id: 1 }, "Chat" }
                }
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn NavbarLayout() -> Element {
    rsx! {
        div { class: "container-fluid",
            Nav {}
            main { Outlet::<Route> {} }
        }
    }
}

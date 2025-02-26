use dioxus::prelude::*;

use crate::pages::*;
use crate::constants::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(NavbarLayout)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

#[component]
fn Nav() -> Element {
    rsx! {
        nav {
            ul {
                li {
                    strong {
                        "{APP_TITLE}"
                    }
                }
            }
            ul {
                li {
                    Link {
                        to: Route::Home {},
                        "Home"
                    }
                }
                li {
                    Link {
                        to: Route::Blog { id: 1 },
                        "Blog"
                    }
                }
            }
        }
    }
}


/// Shared navbar component.
#[component]
fn NavbarLayout() -> Element {
    rsx! {
        div {
            class: "container",
            Nav {}
            main {
                Outlet::<Route> {}
            }
        }
    }
}

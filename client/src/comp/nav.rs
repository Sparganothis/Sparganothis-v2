use crate::constants::*;
use crate::localstorage::LocalStorageContext;
use crate::network::NetworkConnectionStatusIcon;
use crate::route::Route;
use dioxus::prelude::*;

#[component]
pub fn Nav() -> Element {
    let my_nickname = use_context::<LocalStorageContext>().user_secrets;
    let my_nickname = use_memo(move || {
        my_nickname.read().user_identity().nickname().to_string()
    });
    rsx! {
        nav {
            ul {
                li {
                    Link { to: Route::Home {},   strong { "{APP_TITLE}" } }
                }
            }
            ul {
                li {
                    NetworkConnectionStatusIcon {}
                }
            }
            ul {
                li {
                    Link { to: Route::MyProfilePage {}, b {"{my_nickname}"} }
                }
                li {
                    Link { to: Route::GlobalChatPage {  }, "Chat" }
                }
            }
            ul {
                li {
                    a {
                        href: "https://github.com/Sparganothis/Sparganothis-v2",
                        style: "display: flex; flex-direction:row; align-items: center;",
                        GithubIcon {},
                        "GitHub",
                        img {
                            src: "https://github.com/Sparganothis/Sparganothis-v2/actions/workflows/rust.yml/badge.svg",
                            style: "height: 1rem; padding-left: 0.2rem; margin-left: 0.2rem;",
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GithubIcon() -> Element {
    let sstr = include_str!("../../assets/github.svg.html");
    rsx! {
        div {
            style: "width: 1rem; height: 1rem; padding-right: 0.2rem; margin-right: 0.2rem; margin-top: -0.7rem;",
            dangerous_inner_html: sstr,
        }
    }
}

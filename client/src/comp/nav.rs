use crate::constants::*;
use crate::localstorage::LocalStorageContext;
use crate::network::NetworkConnectionStatusIcon;
use crate::route::Route;
use dioxus::html::g::dangerous_inner_html;
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
                        style: "display: flex; flex-direction:row;",
                        GithubIcon {},
                        "GitHub"
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
            style: "width: 1.3rem; height: 1.3rem; padding-right: 0.2rem; margin-right: 0.2rem",
            dangerous_inner_html: sstr,
        }
    }
}
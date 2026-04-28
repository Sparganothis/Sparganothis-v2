use crate::constants::*;
use crate::localstorage::LocalStorageContext;
use crate::network::NetworkConnectionStatusIcon;
use crate::route::Route;
use dioxus::prelude::*;
use tracing::info;

#[component]
pub fn Nav() -> Element {
    info!("Nav");
    let my_secrets =
        use_context::<LocalStorageContext>().persistent.user_secrets;
    let my_nickname = use_memo(move || {
        my_secrets.read().user_identity().nickname().to_string()
    });
    
    // Online count (mocked or from context if available)
    let online_count = 4; 

    rsx! {
        nav {
            ul {
                li {
                    Link { 
                        class: "brand",
                        to: Route::Home {},   
                        "{APP_TITLE}" 
                        span { "Strategic. Competitive. Timeless." }
                    }
                }
            }
            ul {
                li {
                    Link { to: Route::PlaySingleplayerPage { }, "Singleplayer" }
                }
                li {
                    Link { to: Route::MatchmakingPage { }, "1v1 matchmaking" }
                }
            }

            ul {
                li {
                    div { class: "online-indicator", "• {online_count} ONLINE" }
                }
                li {
                    Link { to: Route::GlobalChatPage { }, "Chat" }
                }
                li {
                    Link { to: Route::UsersRootDirectoryPage { }, "Top Players" }
                }
            }
            
            ul {
                li {
                    LinkDropdownProfile{my_nickname}
                }
                li {
                    a {
                        href: "https://github.com/Sparganothis/Sparganothis-v2",
                        target: "_blank",
                        class: "flex items-center gap-1",
                        GithubIcon {},
                        "GitHub",
                        img {
                            src: "https://github.com/Sparganothis/Sparganothis-v2/actions/workflows/rust.yml/badge.svg",
                            style: "height: 1rem;",
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
            class: "github-icon-wrapper",
            dangerous_inner_html: sstr,
        }
    }
}

#[component]
fn LinkDropdownProfile(my_nickname: ReadOnlySignal<String>) -> Element {
    rsx! {
        details {
            class: "dropdown",
            summary {
                b {
                    "{my_nickname}"
                }
            }
            ul {
                li {
                    Link { to: Route::MyProfilePage {}, "My Profile"}
                }
                li {
                    Link { to: Route::MyMainSettings {}, "Game Settings" }
                }
                li {
                    Link { to: Route::MyButtonSettings {}, "Button Settings" }
                }
            }
        }
    }
}

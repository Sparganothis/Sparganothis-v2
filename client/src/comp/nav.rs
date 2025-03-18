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
        }
    }
}

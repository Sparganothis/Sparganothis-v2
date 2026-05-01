use dioxus::prelude::*;
use protocol::user_identity::{NodeIdentity, UserIdentity};

use crate::{
    localstorage::LocalStorageContext,
    network::NetworkState,
    route::{Route, UrlParam},
};

#[component]
pub fn CurrentUserInfoDisplay() -> Element {
    let user = use_context::<LocalStorageContext>().persistent.user_secrets;
    let user_id = use_memo(move || *user.read().user_identity());

    let node_info = use_context::<NetworkState>().global_mm;
    let node_id = use_memo(move || {
        node_info.read().clone().map(|mm| mm.own_node_identity())
    });
    let node_id: ReadSignal<Option<NodeIdentity>> = node_id.into();

    rsx! {
        article {
            style: "
                height: 400px;
                overflow: auto;
            ",
            UserInfoDisplay { info: *user_id.read(), node_id }
            h1 {
                Link {
                    to: Route::MyMainSettings {  },
                    "Game Settings"
                }
            }
            h1 {
                Link {
                    to: Route::MyButtonSettings {   },
                    "Button Settings"
                }
            }
            h1 {
                Link {
                    to: Route::UsersProfilePage { user_id: UrlParam(*user_id.read()) },
                    "Your Public Profile"
                }
            }
        }
    }
}

#[component]
fn UserInfoDisplay(
    info: UserIdentity,
    node_id: ReadSignal<Option<NodeIdentity>>,
) -> Element {
    rsx! {
        div {
            h1 {
                "Nickname: ", i { "{info.nickname()}" }
            }
            h4 {
                "User ID: {info.user_id()}"
            }
            if let Some(node_id) = node_id.read().as_ref() {
                h5 {
                    "Node ID: {node_id.node_id()}"
                }
            }
        }
    }
}

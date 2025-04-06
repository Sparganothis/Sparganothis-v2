use dioxus::prelude::*;
use protocol::user_identity::{NodeIdentity, UserIdentity};

use crate::{localstorage::LocalStorageContext, network::NetworkState, route::Route};

#[component]
pub fn CurrentUserInfoDisplay() -> Element {
    let user = use_context::<LocalStorageContext>().persistent.user_secrets;
    let user_id = use_memo(move || user.read().user_identity().clone());

    let node_info = use_context::<NetworkState>().global_mm;
    let node_id = use_memo(move || {
        if let Some(mm) = node_info.read().clone() {
            Some(mm.own_node_identity().clone())
        } else {
            None
        }
    });
    let node_id: ReadOnlySignal<Option<NodeIdentity>> = node_id.into();
    rsx! {
        article {
            style: "
                height: 400px;
                overflow: auto;
            ",
            UserInfoDisplay { info: user_id.read().clone(), node_id }
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
        }
    }
}

#[component]
pub fn UserInfoDisplay(
    info: UserIdentity,
    node_id: ReadOnlySignal<Option<NodeIdentity>>,
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

#[component]
pub fn UserProfileLink() -> Element {
    rsx! {}
}

use dioxus::prelude::*;
use protocol::user_identity::UserIdentity;

use crate::localstorage::LocalStorageContext;

#[component]
pub fn CurrentUserInfoDisplay() -> Element {
    let user = use_context::<LocalStorageContext>().user_secrets;
    let user_id = use_memo(move || user.read().user_identity().clone());
    rsx! {
        UserInfoDisplay { info: user_id.read().clone() }
    }
}

#[component]
pub fn UserInfoDisplay(info: UserIdentity) -> Element {
    rsx! {
        div {
            h1 {
                "{info.nickname()}"
            }
            h3 {
                "{info.user_id()}"
            }
        }
    }
}

#[component]
pub fn UserProfileLink() -> Element {
    rsx! {}
}

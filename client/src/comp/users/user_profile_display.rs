use dioxus::prelude::*;
use protocol::{
    global_matchmaker::GlobalMatchmaker,
    server_chat_api::client_api_manager::ClientApiManager,
    user_identity::UserIdentity,
};

#[component]
pub fn UserProfileDisplay(
    api: ReadOnlySignal<ClientApiManager>,
    mm: ReadOnlySignal<GlobalMatchmaker>,
    user_id: ReadOnlySignal<UserIdentity>,
) -> Element {
    let nickname = user_id.read().nickname();

    rsx! {
        
        h1 {
            "User \"{nickname}\""
        }
    }
}

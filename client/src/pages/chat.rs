use dioxus::prelude::*;
use protocol::global_matchmaker::GlobalChatMessageType;

use crate::{comp::chat_comp::ChatRoom, network::NetworkState};
/// Blog page
#[component]
pub fn GlobalChatPage() -> Element {
    let mm = use_context::<NetworkState>().global_mm;
    let chat = use_resource(move || {
        let mm = mm.read().clone();
        async move { Some(mm?.global_chat_controller().await?) }
    });
    let chat =
        use_memo(move || chat.read().as_ref().map(|c| c.clone()).flatten());
    let presence = use_context::<NetworkState>().global_presence_list;
    rsx! {
        ChatRoom<GlobalChatMessageType> { chat, presence }
    }
}

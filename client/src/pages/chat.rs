use dioxus::prelude::*;
use protocol::global_matchmaker::GlobalChatMessageType;

use crate::comp::chat::{chat_signals_hook::use_global_chat_controller_signal, chat_window_fullscreen::FullscreenChatRoom};
/// Blog page
#[component]
pub fn GlobalChatPage() -> Element {
    let chat = use_global_chat_controller_signal();
    rsx! {
        FullscreenChatRoom<GlobalChatMessageType> { chat }
    }
}

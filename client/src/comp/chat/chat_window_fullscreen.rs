use super::{chat_signals_hook::ChatSignals, chat_traits::ChatMessageType};
use crate::comp::chat::{
    chat_display::{ChatHistoryDisplay, ChatPresenceDisplay},
    chat_input::ChatInput,
};
use dioxus::prelude::*;
use tracing::info;

#[component]
pub fn FullscreenChatRoom<T: ChatMessageType>(chat: ChatSignals<T>) -> Element {
    info!("FullscreenChatRoom");
    let presence = chat.presence;
    let history = chat.history;
    let on_user_message = chat.send_broadcast_user_message;

    rsx! {
        div {
            class: "chat-window-container",
            div {
                class: "chat-left-pane",
                ChatPresenceDisplay::<T> { presence }
            }
            div {
                class: "chat-main-pane",
                div {
                    class: "chat-main",
                    ChatHistoryDisplay::<T> { history }
                }
                div {
                    class: "chat-bottom",
                    ChatInput::<T> { on_user_message }
                }
            }
        }
    }
}

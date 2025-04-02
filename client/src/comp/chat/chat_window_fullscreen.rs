use super::{chat_signals_hook::{ChatControllerSignal, ChatSignals}, chat_traits::ChatMessageType};
use crate::comp::chat::{
            chat_display::{ChatHistoryDisplay, ChatPresenceDisplay},
            chat_input::ChatInput, chat_signals_hook::{use_chat_history_signal, use_chat_message_callback, use_chat_presence_signal},
        };
use dioxus::prelude::*;

#[component]
pub fn FullscreenChatRoom<T: ChatMessageType>(
    chat: ChatControllerSignal<T>,
) -> Element {
    let presence = use_chat_presence_signal(chat);
    let history = use_chat_history_signal(chat);
    let on_user_message = use_chat_message_callback(chat, Some(history));

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

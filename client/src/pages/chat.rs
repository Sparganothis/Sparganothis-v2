use dioxus::prelude::*;
use iroh::SecretKey;
use protocol::global_matchmaker::GlobalMatchmaker;
use tracing::info;
use uuid::Uuid;

use crate::{localstorage::LocalStorageContext, route::Route};
/// Blog page
#[component]
pub fn ChatPage(id: i8) -> Element {
    let user_uuid = use_context::<LocalStorageContext>().user_id;
    rsx! {
        div { id: "blog",
            h1 { "This is Chatroom #{id}!" }
            h3 { "User ID = {user_uuid}" }

            ChatRoom { id }

            Link { to: Route::ChatPage { id: id.wrapping_add(-1) }, "Previous" }
            span { " <---> " }
            Link { to: Route::ChatPage { id: id.wrapping_add(1) }, "Next" }
        }
    }
}

#[component]
fn ChatRoom(id: ReadOnlySignal<i8>) -> Element {
    let mut history = use_signal(ChatHistory::default);
    use_effect(move || {
        let _i = id.read();
        *history.write() = ChatHistory::default();
    });
    let chatroom_id = *id.read();
    rsx! {
        div {
            ChatHistoryDisplay { chatroom_id, history }
            ChatInput { chatroom_id, history }
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
struct ChatMessage {
    pub user_id: Uuid,
    pub chatroom_id: i8,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Default)]
struct ChatHistory {
    pub messages: Vec<ChatMessage>,
}

#[component]
fn ChatHistoryDisplay(chatroom_id: i8, history: ReadOnlySignal<ChatHistory>) -> Element {
    rsx! {
        article {
            for message in history.read().messages.iter() {
                ChatMessageDisplay { message: message.clone() }
            }
            if history.read().messages.is_empty() {
                p {
                    "No messages."
                }
            }
        }
    }
}

#[component]
fn ChatMessageDisplay(message: ChatMessage) -> Element {
    rsx! {
        article {
            h3 { "Message from {message.user_id}" }
            p { "{message.message}" }
            p { "{message.timestamp.to_rfc3339()}" }
        }
    }
}

#[component]
fn ChatInput(chatroom_id: i8, history: Signal<ChatHistory>) -> Element {
    let user_uuid = use_context::<LocalStorageContext>().user_id;
    let mut message_input = use_signal(String::new);
    let mut send_message = move || {
        let mut _i = message_input.write();
        let message = _i.clone();
        let message = ChatMessage {
            user_id: user_uuid.read().clone(),
            chatroom_id: chatroom_id,
            message,
            timestamp: chrono::Utc::now(),
        };
        do_send_message(message.clone());
        history.write().messages.push(message);
        _i.clear();
    };
    rsx! {
        article {
            input {
                value: "{message_input.read()}",
                oninput: move |e| {
                    *message_input.write() = e.value();
                },
                onkeyup: move |e| {
                    if e.key() == Key::Enter {
                        send_message();
                    }
                }
            }
            button { onclick: move |_| send_message(), "Send" }

        }
    }
}

fn do_send_message(message: ChatMessage) {
    info!("Sending message...");
}

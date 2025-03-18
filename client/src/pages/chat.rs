use dioxus::prelude::*;
use n0_future::StreamExt;
use protocol::{_const::GLOBAL_CHAT_TOPIC_ID, chat::{ChatEvent, ChatEventStreamError}, global_matchmaker::GlobalMatchmaker};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{localstorage::LocalStorageContext, network::NetworkState, route::Route};
/// Blog page
#[component]
pub fn GlobalChatPage() -> Element {
    rsx! {
        ChatRoom { id: GLOBAL_CHAT_TOPIC_ID.to_string() }
    }
}

#[component]
fn ChatRoom(id: ReadOnlySignal<String>) -> Element {
    let mut history = use_signal(ChatHistory::default);
    let mm = use_context::<NetworkState>().global_mm;
    let _ = use_resource(move || {
        let mm =  mm.read().clone();
        async move {
            let Some(mm) = mm else {
                return;
            };
            let Some(controller) = mm.global_chat_controller().await else {
                return;
            };
            let mut recv = controller.receiver();
            while let Some(event) = recv.next().await {
                history.write().messages.push(event.map_err(|e| e.to_string()));
            }
    }});

    use_effect(move || {
        let _i = id.read().clone();
        let _i2 = use_context::<NetworkState>().is_connected.read().clone();
        *history.write() = ChatHistory::default();
    });
    let chatroom_id = id.read().clone();
    rsx! {
        div {
            class: "chat-window-container",
            div {
                class: "chat-left-pane",
            }
            div {
                class: "chat-main-pane",
                div {
                    class: "chat-main",
                    ChatHistoryDisplay { chatroom_id: chatroom_id.clone(), history }
                }
                div {
                    class: "chat-bottom",
                    ChatInput { chatroom_id: chatroom_id.clone(), history }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
struct ChatHistory {
    pub messages: Vec<Result<ChatEvent, String>>,
}

#[component]
fn ChatHistoryDisplay(chatroom_id: String, history: ReadOnlySignal<ChatHistory>) -> Element {
    rsx! {
        div {
            style: "
                height: 100%;
                overflow: hidden;
            ",
            article {
                style: "
                    height: 100%;
                    overflow: scroll;
                ",
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
}

#[component]
fn ChatMessageDisplay(message: Result<ChatEvent, String>) -> Element {
    rsx! {
        article {
            onmounted: move |_e| async move {
                let _e = _e.scroll_to(ScrollBehavior::Smooth).await;
                if let Err(e) = _e {
                    warn!("Failed to scroll to bottom: {}", e);
                }
            },
            pre {
                "{message:#?}"
            }
        }
    }
}

#[component]
fn ChatInput(chatroom_id: String, history: Signal<ChatHistory>) -> Element {
    let mut message_input = use_signal(String::new);
    let userinfo = use_context::<LocalStorageContext>().user_secrets.read().clone();
    let nickname = userinfo.user_identity().nickname().to_string();
    let userid = userinfo.user_identity().user_id().clone();

    let mm = use_context::<NetworkState>().global_mm;
    let send_message = Callback::new(move |_:()| {
        let mut _i = message_input.write();
        let message = _i.clone();
        _i.clear();
        chat_send_message(mm.clone(), chatroom_id.clone(), message.clone());
        history.write().messages.push(Ok(ChatEvent::MessageReceived {
            node_id:  userid.clone() ,
            text: message.clone(),
            nickname:nickname.clone(),
            sent_timestamp: 0,
        }));
    });
    rsx! {
        article {
            role: "group",
            input {
                value: "{message_input.read()}",
                oninput: move |e| {
                    *message_input.write() = e.value();
                },
                onkeyup: move |e| {
                    if e.key() == Key::Enter {
                        send_message.call(());
                    }
                }
            }
            button { onclick: move |_| send_message.call(()), "Send" }
        }
    }
}

fn chat_send_message(mm: ReadOnlySignal<Option<GlobalMatchmaker>>, _chatroom_id: String, message: String) {
    let Some(mm) = mm.read().clone() else {
        return;
    };
    spawn(async move {
        let Some(controller) = mm.global_chat_controller().await else {
            return;
        };
        let sender = controller.sender();
        match sender.send(message).await {
            Ok(_) => (),
            Err(e) => {
                warn!("Failed to send message: {}", e);
            }
        }
    });
}
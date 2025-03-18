use std::{collections::BTreeMap, time::Duration};
use n0_future::time::Instant;

use dioxus::prelude::*;
use iroh::{NodeId, PublicKey};
use n0_future::StreamExt;
use protocol::{
    _const::{GLOBAL_CHAT_TOPIC_ID, PRESENCE_INTERVAL},
    chat::{timestamp_now, ChatController, ChatEventStreamError, ChatMessage, NetworkEvent, ReceivedMessage},
    global_matchmaker::GlobalMatchmaker, user_identity::NodeIdentity,
};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{localstorage::LocalStorageContext, network::NetworkState, route::Route};
/// Blog page
#[component]
pub fn GlobalChatPage() -> Element {
    let mm = use_context::<NetworkState>().global_mm;
    let chat = use_resource(move || {
        let mm = mm.read().clone();
        async move {
            Some(mm?.global_chat_controller().await?) 
        }
    });
    rsx! {
        ChatRoom { chat }
    }
}

#[component]
fn ChatRoom(chat: ReadOnlySignal<Option<Option<ChatController>>>) -> Element {
    let mut history = use_signal(ChatHistory::default);
    let mm = use_context::<NetworkState>().global_mm;
    let mut presence = use_signal(ChatPresenceData::default);

    let _ = use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            n0_future::time::sleep(Duration::from_secs(1)).await;
            presence.write().remove_expired();
            if let Some(m) = mm.peek().as_ref() {
                presence.write().add_presence(m.own_node_identity());
            }
        }
    });
    let _ = use_resource(move || {
        let mm = mm.read().clone();
        let chat = chat.read().clone();
        async move {
            let Some(_mm) = mm else {
                return;
            };
            let Some(Some(controller)) = chat else {
                return;
            };
            let mut recv = controller.receiver();
            while let Some(event) = recv.next().await {
                let t: Result<ReceivedMessage, String>= match event {
                    Ok(NetworkEvent::Message { event }) => {
                        match event.message {
                            ChatMessage::Presence {  } => {
                                presence.write().add_presence(event.from);
                                continue;
                            },
                            _ => Ok(event),
                        }
                    },
                    Err(e) => {
                        Err(e.to_string())
                    },
                    _ => {
                        continue;
                    }
                };
                history
                    .write()
                    .messages
                    .push(t);
            }
        }
    });

    use_effect(move || {
        let _i2 = use_context::<NetworkState>().is_connected.read().clone();
        *history.write() = ChatHistory::default();
        *presence.write() = ChatPresenceData::default();
    });
    let on_user_message = Callback::new(move |message: String| {
        let m = chat_send_message(mm.clone(), message);
        if let Some(m) = &m {
            history
                .write()
                .messages
                .push(Ok(m.clone()));
        } else {
            history.write().messages.push(Err("Failed to send message".to_string()));
        }
        m
    });
    rsx! {
        div {
            class: "chat-window-container",
            div {
                class: "chat-left-pane",
                ChatPresenceDisplay { presence }
            }
            div {
                class: "chat-main-pane",
                div {
                    class: "chat-main",
                    ChatHistoryDisplay { history }
                }
                div {
                    class: "chat-bottom",
                    ChatInput { on_user_message }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
struct ChatHistory {
    pub messages: Vec<Result<ReceivedMessage, String>>,
}

#[derive(Clone, Debug, PartialEq, Default)]
struct ChatPresenceData {
    pub presence: BTreeMap<NodeId, (Instant, NodeIdentity)>,
}

impl ChatPresenceData {
    pub fn add_presence(&mut self,  identity: NodeIdentity) {
        self.presence.insert(identity.node_id().clone(), (Instant::now(), identity));
    }
    pub fn remove_expired(&mut self) {
        let now = Instant::now();
        self.presence.retain(|_, (last_seen, _)| {
            now.duration_since(*last_seen) < PRESENCE_INTERVAL * 3
        });
    }
}

#[component]
fn ChatPresenceDisplay(presence: ReadOnlySignal<ChatPresenceData>) -> Element {
    let presence = use_memo(move || {
        let mut p = presence.read().presence.clone().into_iter().collect::<Vec<_>>();
        p.sort_by_key(|(_, (_k, _userid))| (_userid.user_id().to_string(), _userid.nickname().to_string()) );
        p
    });
    rsx! {
        ul {
            for (node_id, (last_seen, identity)) in presence.read().iter() {
                li {
                    key: "{node_id}",
                    ChatPresenceDisplayItem { last_seen: last_seen.clone(), identity: identity.clone() }
                }
            }
            if presence.read().is_empty() {
                p {
                    "No presence data."
                }
            }
        }
    }
}

#[component]
fn ChatPresenceDisplayItem(last_seen: ReadOnlySignal<Instant>, identity: ReadOnlySignal<NodeIdentity>) -> Element {
    let mut last_seen_txt = use_signal(|| "".to_string());
    let _ = use_resource(move || async move {
        loop {
            let elapsed = 1 + last_seen.peek().elapsed().as_secs();
            let elapsed_txt = pretty_duration::pretty_duration(&Duration::from_secs(elapsed), None);
            last_seen_txt.set(format!("{} ago", elapsed_txt));
            let wait = 1 + elapsed / 10;
            n0_future::time::sleep(Duration::from_secs(wait)).await;
        }
    });
    let identity = identity.read().clone();
    rsx! {
        div {
            style: "width: 90%;",
            "data-tooltip": "
                {identity.user_id().fmt_short()}@{identity.node_id().fmt_short()}
                (last seen: {last_seen_txt})
            ",
            "data-placement": "bottom",
            "{identity.nickname()}"
        }    
    }
}
#[component]
fn ChatHistoryDisplay(history: ReadOnlySignal<ChatHistory>) -> Element {
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
                    ChatMessageOrErrorDisplay { message: message.clone() }
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
fn ChatMessageOrErrorDisplay(message: Result<ReceivedMessage, String>) -> Element {
    let mm = use_context::<NetworkState>().global_mm;
    let Some(mm) = mm.read().clone() else {
        return rsx! {}
    };
    let user_id = mm.user().user_id().clone();
    match message {
        Ok(message) => rsx!{
            ChatMessageDisplay { message, user_id}
        },
        Err(err) => rsx! {
            pre {
                "{err}"
            }
        }
    }
}

#[component]
fn ChatMessageDisplay(message: ReceivedMessage, user_id: PublicKey) -> Element {
    let ReceivedMessage { timestamp, from, message } = message;
    let text = match message {
        ChatMessage::Message { text } => text,
        _ => format!("{message:#?}"),
    };
    let from_nickname = from.nickname();
    let from_user_id = from.user_id().fmt_short();
    let from_node_id = from.node_id().fmt_short();
    let align = if from.user_id() != &user_id {
        "left"
    } else {
        "right"
    };

    let mut last_seen_txt = use_signal(|| "".to_string());
    let _ = use_resource(move || async move {
        loop {
            let elapsed = (1 + timestamp_now().timestamp() - timestamp.timestamp()).abs() as u64;
            let elapsed_txt = pretty_duration::pretty_duration(&Duration::from_secs(elapsed), None);
            last_seen_txt.set(format!("{} ago", elapsed_txt));
            let wait = 1 + elapsed / 10;
            n0_future::time::sleep(Duration::from_secs(wait)).await;
        }
    });

    rsx! {
        div {
            style: "width: 100%; height: fit-content; display: flex; justify-content: {align};",
            article {
                style: "max-width: 70%; min-width: 30%; width: fit-content; text-align: {align}; float: {align};",
                onmounted: move |_e| async move {
                    let _e = _e.scroll_to(ScrollBehavior::Smooth).await;
                    if let Err(e) = _e {
                        warn!("Failed to scroll to bottom: {}", e);
                    }
                },
                header {
                    style: "display: flex; justify-content: space-between;",
                    span {
                        "{from_nickname}"
                    }
                    small {
                        "{from_user_id}@{from_node_id}"
                    }
                }
                p {
                    "{text}"
                }
                footer {
                    style: "padding-top: 0px; margin-top: 0px;",
                    small {
                        "{last_seen_txt}"
                    }
                }
            }
        }
    }
}

#[component]
fn ChatInput(on_user_message: Callback<String, Option<ReceivedMessage>>) -> Element {
    let mut message_input = use_signal(String::new);
    let is_connected = use_context::<NetworkState>().is_connected;

    let send_message = Callback::new(move |_: ()| {
        let mut _i = message_input.write();
        let message = _i.clone();
        let m = on_user_message.call(message.clone());
        if let Some(_m) = m {
            _i.clear();
        } else {
            warn!("Failed to send message");
        }
    });
    let disabled = use_memo(move || {
        let m = message_input.read().clone();
        let is_connected = is_connected.read().clone();
        if m.trim().len() <= 2 {
            return true;
        }
        if !is_connected {
            return true;
        };
        false
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
            button { onclick: move |_| send_message.call(()), disabled: disabled,  "Send" }
        }
    }
}

fn chat_send_message(
    mm: ReadOnlySignal<Option<GlobalMatchmaker>>,
    message: String,
) -> Option<ReceivedMessage> {
    let Some(mm) = mm.peek().clone() else {
        return None;
    };
    let from = mm.own_node_identity().clone();
    let ts = timestamp_now();
    let msg_preview = ReceivedMessage {
        timestamp: ts,
        from,
        message: ChatMessage::Message { text: message.clone() },
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
    Some(msg_preview)
}

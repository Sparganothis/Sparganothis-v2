use n0_future::time::Instant;
use std::{collections::VecDeque, time::Duration};

use dioxus::prelude::*;
use iroh::PublicKey;
use protocol::{
    chat::{ChatController, IChatController, IChatReceiver, IChatSender},
    chat_presence::{PresenceFlag, PresenceList},
    datetime_now,
    global_matchmaker::{
        GlobalChatMessageType, GlobalChatPresence, GlobalMatchmaker,
    },
    user_identity::NodeIdentity,
    IChatRoomType as ChatMessageType2, ReceivedMessage,
};
use tracing::warn;
use crate::network::NetworkState;

pub trait ChatMessageType:
    ChatMessageType2 + RenderElement + FromUserInput + Clone
{
}
impl<T> ChatMessageType for T where
    T: ChatMessageType2 + RenderElement + FromUserInput + Clone
{
}
pub trait RenderElement: ChatMessageType2 {
    fn render_message(message: <Self as ChatMessageType2>::M) -> Element;
    fn render_presence(
        payload: Option<<Self as ChatMessageType2>::P>,
    ) -> Element;
}
pub trait FromUserInput: ChatMessageType2 {
    fn from_user_input(input: String) -> <Self as ChatMessageType2>::M;
}

impl FromUserInput for GlobalChatMessageType {
    fn from_user_input(input: String) -> <Self as ChatMessageType2>::M {
        input
    }
}
impl RenderElement for GlobalChatMessageType {
    fn render_message(message: <Self as ChatMessageType2>::M) -> Element {
        rsx! {
                "{message}"
        }
    }
    fn render_presence(payload: Option<GlobalChatPresence>) -> Element {
        match payload {
            Some(payload) => {
                rsx! {
                    br{}
                    if ! payload.platform.is_empty() {
                        small {
                            "{payload.platform}:"
                        }
                    }
                    if ! payload.url.is_empty() {
                        small {
                            "{payload.url}"
                        }
                    }
                }
            }
            None => rsx! {
                br{}
            },
        }
    }
}


#[component]
pub fn ChatRoom<T: ChatMessageType>(
    chat: ReadOnlySignal<Option<ChatController<T>>>,
    presence: ReadOnlySignal<PresenceList<T>>,
) -> Element {
    let mut history = use_signal(ChatHistory::<T>::default);
    let mm = use_context::<NetworkState>().global_mm;

    let _ = use_resource(move || {
        let mm = mm.read().clone();
        let chat = chat.read().clone();
        async move {
            let Some(_mm) = mm else {
                return;
            };
            let Some(controller) = chat else {
                return;
            };
            let recv = controller.receiver().await;
            while let Some(message) = recv.next_message().await {
                history.write().push(Ok(message));
            }
            warn!("XXX: ChatRoom receiver stream closed");
        }
    });

    use_effect(move || {
        let _i2 = use_context::<NetworkState>().is_connected.read().clone();
        *history.write() = ChatHistory::<T>::default();
    });
    let on_user_message = Callback::new(move |message: T::M| {
        let m = chat_send_message(mm.clone(), chat.clone(), message);
        if let Some(m) = &m {
            history.write().push(Ok(m.clone()));
        } else {
            history
                .write()
                .push(Err("Failed to send message".to_string()));
        }
        m
    });
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

#[derive(Clone, Debug, PartialEq)]
struct ChatHistory<T: ChatMessageType> {
    messages: VecDeque<Result<ReceivedMessage<T>, String>>,
}
impl<T: ChatMessageType> Default for ChatHistory<T> {
    fn default() -> Self {
        Self { messages: VecDeque::new() }
    }
}

impl<T: ChatMessageType> ChatHistory<T> {
    fn push(&mut self, item: Result<ReceivedMessage<T>, String>) {
        const UI_MAX_MESSAGE_COUNT: usize = 16;
        self.messages.push_back(item);
        if self.messages.len() > UI_MAX_MESSAGE_COUNT {
            self.messages.pop_front();
        }
    }
}

#[component]
fn ChatPresenceDisplay<T: ChatMessageType>(
    presence: ReadOnlySignal<PresenceList<T>>,
) -> Element {
    rsx! {
        ul {
            for (presence_flag,last_seen, identity, payload, rtt) in presence.read().iter() {
                ChatPresenceDisplayItem::<T> {
                    presence_flag: presence_flag.clone(),
                    last_seen: last_seen.clone(),
                    identity: identity.clone(),
                    payload: payload.clone() as Option<T::P>,
                    rtt: rtt.clone(),
                }
            }
            if presence.read().is_empty() {
                i {
                    "No presence data."
                }
            }
        }
    }
}

#[component]
fn ChatPresenceDisplayItem<T: ChatMessageType>(
    presence_flag: ReadOnlySignal<PresenceFlag>,
    last_seen: ReadOnlySignal<Instant>,
    identity: ReadOnlySignal<NodeIdentity>,
    payload: ReadOnlySignal<Option<T::P>>,
    rtt: ReadOnlySignal<Option<u16>>,
) -> Element {
    let mut last_seen_txt = use_signal(|| "".to_string());
    let payload = use_memo(move || T::render_presence(payload.read().clone()));
    let mm = use_context::<NetworkState>().global_mm;
    let mut own_node_id = use_signal(|| None);
    let _ = use_resource(move || {
        let mm = mm.read().clone();
        async move {
            let Some(mm) = mm else {
                return;
            };
            own_node_id.set(Some(mm.own_node_identity().node_id().clone()));
            loop {
                let elapsed = 1 + last_seen.peek().elapsed().as_secs();
                let elapsed_txt = pretty_duration::pretty_duration(
                    &Duration::from_secs(elapsed),
                    None,
                );
                last_seen_txt.set(format!("{} ago", elapsed_txt));
                let wait = 1 + elapsed / 10;
                mm.sleep(Duration::from_secs(wait)).await;
            }
        }
    });
    let is_own_node = use_memo(move || {
        let own_node_id = own_node_id.read().clone();
        let identity = identity.read().clone();
        own_node_id == Some(identity.node_id().clone())
    });
    let identity = identity.read().clone();
    let own_color = identity.color();

    let color = match presence_flag.read().clone() {
        PresenceFlag::ACTIVE => "darkgreen",
        PresenceFlag::IDLE => "orange",
        PresenceFlag::EXPIRED => "darkred",
        PresenceFlag::UNCONFIRMED => "red",
    };
    let element = rsx! {
        "{identity.nickname()}",
        {payload}
    };
    let element = if identity.bootstrap_idx().is_some() {
        rsx! {
            small {
                {element}
            }
        }
    } else {
        element
    };
    rsx! {
        li {
            key: "{identity.node_id()}",
            style: "width: calc(90%-30px); color: {color}; position: relative;",
            "data-tooltip": "
                {identity.user_id().fmt_short()}@{identity.node_id().fmt_short()}
                (last seen: {last_seen_txt})
            ",
            "data-placement": "bottom",
            {element}
            small { small {
                style: "float: right; color: #666;",
                if let Some(rtt) = rtt.read().clone() {
                    "{rtt} ms"
                } else if is_own_node.read().clone() {
                    "(you)"
                }
            }}
            div {
                style: "
                left: -2.1rem;
                top: 0.5rem;
                position:absolute;
                ",
                ChatUserPortraitBox {  own_color: own_color }
            }
        }
    }
}

#[component]
fn ChatUserPortraitBox (
    own_color: ReadOnlySignal<String>,
) -> Element {
    rsx! {
        div {
            style: "
            width: 1.8rem;
            height: 1.8rem;
            border: 0.5rem solid {own_color};
            z-index:1;
            "
        }
    }
}

#[component]
fn ChatHistoryDisplay<T: ChatMessageType>(
    history: ReadOnlySignal<ChatHistory<T>>,
) -> Element {
    rsx! {
        div {
            style: "
                height: 100%;
                overflow: hidden;
            ",
            article {
                style: "
                    height: 100%;
                    overflow-y: auto;
                    overflow-x: hidden;
                ",
                for message in history.read().messages.iter() {
                    ChatMessageOrErrorDisplay::<T> { message: message.clone() }
                }
                if history.read().messages.is_empty() {
                    i {
                        "No messages."
                    }
                }
            }
        }
    }
}

#[component]
fn ChatMessageOrErrorDisplay<T: ChatMessageType>(
    message: Result<ReceivedMessage<T>, String>,
) -> Element {
    let mm = use_context::<NetworkState>().global_mm;
    let Some(mm) = mm.read().clone() else {
        return rsx! {};
    };
    let my_user_id = mm.user().user_id().clone();
    match message {
        Ok(message) => rsx! {
            ChatMessageDisplay::<T> { message, my_user_id}
        },
        Err(err) => rsx! {
            pre {
                "{err}"
            }
        },
    }
}

#[component]
fn ChatMessageDisplay<T: ChatMessageType>(
    message: ReceivedMessage<T>,
    my_user_id: PublicKey,
) -> Element {
    let ReceivedMessage {
        _sender_timestamp: _,
        _received_timestamp: timestamp,
        _message_id,
        from,
        message: text,
    } = message;

    let text = T::render_message(text);

    let from_nickname = from.nickname();
    let from_user_id = from.user_id().fmt_short();
    let from_node_id = from.node_id().fmt_short();
    let from_color = from.color();
    let align = if from.user_id() != &my_user_id {
        "left"
    } else {
        "right"
    };

    let mut last_seen_txt = use_signal(|| "".to_string());
    let mm = use_context::<NetworkState>().global_mm;
    let _ = use_resource(move || {
        let mm = mm.read().clone();
        async move {
            let Some(mm) = mm else {
                return;
            };
            loop {
                let elapsed = (1 + datetime_now().timestamp()
                    - timestamp.timestamp())
                .abs() as u64;
                let elapsed_txt = pretty_duration::pretty_duration(
                    &Duration::from_secs(elapsed),
                    None,
                );
                last_seen_txt.set(format!("{} ago", elapsed_txt));
                let wait = 1 + elapsed / 10;
                mm.sleep(Duration::from_secs(wait)).await;
            }
        }
    });

    rsx! {
        div {
            key: "{_message_id:?}",
            style: "width: 100%; height: fit-content; display: flex; justify-content: {align};",
            article {
                style: "
                    max-width: 70%;
                    min-width: 30%; 
                    width: fit-content; 
                    text-align: {align}; 
                    float: {align};
                    padding: 10px;
                    margin: 10px;
                ",
                onmounted: move |_e| async move {
                    let _e = _e.scroll_to(ScrollBehavior::Instant).await;
                    if let Err(e) = _e {
                        warn!("Failed to scroll to bottom: {}", e);
                    }
                },
                header {
                    style: "display: flex; justify-content: space-between;",
                    b {
                        "{from_nickname}"
                    }
                    small {
                        style: "color: #666;",
                        "{from_user_id}@{from_node_id}"
                    }
                }
                p {
                    style: "position:relative;",
                    {text},
                    div {
                        style: "
                        {align}: 0rem;
                        top: -1.8rem;
                        position:absolute;
                        ",
                        ChatUserPortraitBox {  own_color: from_color }
                    }
                }
                footer {
                    style: "padding-top: 0px; margin-top: 0px; color: #666;",
                    small {
                        "{last_seen_txt}"
                    }
                }
            }
        }
    }
}

#[component]
fn ChatInput<T: ChatMessageType>(
    on_user_message: Callback<T::M, Option<ReceivedMessage<T>>>,
) -> Element {
    let mut message_input = use_signal(String::new);
    let is_connected = use_context::<NetworkState>().is_connected;

    let send_message = Callback::new(move |_: ()| {
        let mut _i = message_input.write();
        let message = _i.clone();
        let message = T::from_user_input(message);
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
        if m.trim().len() < 1 {
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
                        if *disabled.read() {
                            e.prevent_default();
                            return;
                        }
                        send_message.call(());
                    }
                }
            }
            button { onclick: move |_| send_message.call(()), disabled: disabled,  "Send" }
        }
    }
}

fn chat_send_message<T: ChatMessageType>(
    mm: ReadOnlySignal<Option<GlobalMatchmaker>>,
    chat: ReadOnlySignal<Option<ChatController<T>>>,
    message: T::M,
) -> Option<ReceivedMessage<T>> {
    let Some(mm) = mm.peek().clone() else {
        return None;
    };
    let msg_preview = ReceivedMessage {
        _sender_timestamp: datetime_now(),
        _received_timestamp: datetime_now(),
        _message_id: uuid::Uuid::new_v4(),
        from: mm.own_node_identity(),
        message: message.clone(),
    };
    spawn(async move {
        let Some(controller) = chat.peek().clone() else {
            return;
        };
        let sender = controller.sender();
        match sender.broadcast_message(message).await {
            Ok(_) => (),
            Err(e) => {
                warn!("Failed to send message: {}", e);
            }
        }
    });
    Some(msg_preview)
}

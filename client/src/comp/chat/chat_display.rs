use n0_future::time::Instant;
use std::time::Duration;

use crate::network::NetworkState;
use dioxus::prelude::*;
use iroh::PublicKey;
use protocol::{
    chat_presence::{PresenceFlag, PresenceList, PresenceListItem},
    datetime_now,
    user_identity::NodeIdentity,
    ReceivedMessage,
};
use tracing::warn;

use super::{chat_signals_hook::ChatHistory, chat_traits::ChatMessageType};

#[component]
pub fn ChatPresenceDisplay<T: ChatMessageType>(
    presence: ReadOnlySignal<PresenceList<T>>,
) -> Element {
    rsx! {
        ul {
            for PresenceListItem{presence_flag,last_seen, identity, payload, rtt} in presence.read().iter() {
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
    let own_color = identity.html_color();

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
fn ChatUserPortraitBox(own_color: ReadOnlySignal<String>) -> Element {
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
pub fn ChatHistoryDisplay<T: ChatMessageType>(
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
    // let from_user_id = from.user_id().fmt_short();
    // let from_node_id = from.node_id().fmt_short();
    let from_color = from.html_color();
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
                        style: "padding-top: 0px; margin-top: 0px; color: #666;",
                        small {
                            "{last_seen_txt}"
                        }
                    }
                }
                p {
                    style: "position:relative;",
                    div {
                        style: "
                        padding-{align}: 3rem;
                        ",
                        {text},
                    }
                    div {
                        style: "
                        {align}: 0rem;
                        top: -0.2rem;
                        position:absolute;
                        ",
                        ChatUserPortraitBox {  own_color: from_color }
                    }
                }
                // footer {                }
            }
        }
    }
}

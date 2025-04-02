use std::{collections::VecDeque, future::Future};

use dioxus::prelude::*;
use protocol::{
    chat::{ChatController, IChatController, IChatReceiver, IChatSender},
    chat_presence::PresenceList,
    datetime_now,
    global_matchmaker::{GlobalChatMessageType, GlobalMatchmaker},
    IChatRoomType, ReceivedMessage,
};
use tracing::warn;

use crate::network::NetworkState;

use super::chat_traits::ChatMessageType;

#[derive(Clone, Debug, PartialEq)]
pub struct ChatHistory<T: ChatMessageType> {
    pub messages: VecDeque<Result<ReceivedMessage<T>, String>>,
}
impl<T: ChatMessageType> Default for ChatHistory<T> {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }
}

impl<T: ChatMessageType> ChatHistory<T> {
    pub fn push(&mut self, item: Result<ReceivedMessage<T>, String>) {
        const UI_MAX_MESSAGE_COUNT: usize = 16;
        self.messages.push_back(item);
        if self.messages.len() > UI_MAX_MESSAGE_COUNT {
            self.messages.pop_front();
        }
    }
}

pub fn chat_send_message<T: ChatMessageType>(
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

pub type ChatControllerSignal<T> = ReadOnlySignal<Option<ChatController<T>>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChatSignals<T: ChatMessageType> {
    pub chat: ChatControllerSignal<T>,
    pub presence: ReadOnlySignal<PresenceList<T>>,
    pub history: Signal<ChatHistory<T>>,
    pub on_user_message: Callback<T::M, Option<ReceivedMessage<T>>>,
}

pub fn use_chat_signals<
    T: ChatMessageType,
    Fut: 'static + Future<Output = Option<ChatController<T>>>,
>(
    get_chat_fn: Callback<GlobalMatchmaker, Fut>,
) -> ChatControllerSignal<T> {
    let mm = use_context::<NetworkState>().global_mm;
    let chat = use_resource(move || {
        let mm = mm.read().clone();
        async move {
            match mm {
                None => None,
                Some(mm) => get_chat_fn.call(mm).await,
            }
        }
    });
    let chat =
        use_memo(move || chat.read().as_ref().map(|c| c.clone()).flatten());
    chat.into()
}

pub fn use_chat_presence_signal<T: ChatMessageType>(
    chat: ChatControllerSignal<T>,
) -> ReadOnlySignal<PresenceList<T>> {
    let mut presence_list_w = use_signal(move || PresenceList::<T>::default());
    let presence_list = use_memo(move || presence_list_w.read().clone());
    let _poll_presence = use_resource(move || {
        let cc = chat.read().clone();
        async move {
            let Some(cc) = cc else {
                return;
            };
            let presence = cc.chat_presence();
            loop {
                presence_list_w.set(presence.get_presence_list().await);
                presence.notified().await;
            }
        }
    });
    presence_list.into()
}

pub fn use_chat_history_signal<T: ChatMessageType>(
    chat: ChatControllerSignal<T>,
) -> Signal<ChatHistory<T>> {
    let mut history = use_signal(ChatHistory::<T>::default);

    let _record_messages_to_history = use_resource(move || {
        let chat = chat.read().clone();
        async move {
            let Some(cc) = chat else {
                return;
            };
            let recv = cc.receiver().await;
            while let Some(message) = recv.next_message().await {
                history.write().push(Ok(message));
            }
            warn!("XXX: ChatRoom receiver stream closed");
        }
    });
    history
}

pub fn use_chat_message_callback<T: ChatMessageType>(
    chat: ChatControllerSignal<T>,
    history: Option<Signal<ChatHistory<T>>>,
) -> Callback<T::M, Option<ReceivedMessage<T>>> {
    let mm = use_context::<NetworkState>().global_mm;
    let on_user_message =
        Callback::new(move |message: <T as IChatRoomType>::M| {
            let m = chat_send_message(mm.clone(), chat.into(), message);
            if let Some(m) = &m {
                if let Some(mut history) = history {
                    history.write().push(Ok(m.clone()));
                }
            } else {
                if let Some(mut history) = history {
                    history
                        .write()
                        .push(Err("Failed to send message".to_string()));
                }
            }
            m
        });
    on_user_message
}

pub fn use_global_chat_controller_signal(
) -> ChatControllerSignal<GlobalChatMessageType> {
    use_chat_signals(Callback::new(move |mm: GlobalMatchmaker| async move {
        Some(mm.global_chat_controller().await?)
    }))
}

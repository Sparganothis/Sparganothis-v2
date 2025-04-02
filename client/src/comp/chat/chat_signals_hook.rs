use std::{collections::VecDeque, future::Future};

use dioxus::prelude::*;
use n0_future::StreamExt;
use protocol::{
    chat::{ChatController, IChatController, IChatReceiver, IChatSender},
    chat_presence::PresenceList,
    datetime_now,
    global_matchmaker::{GlobalChatMessageType, GlobalMatchmaker},
    IChatRoomType, ReceivedMessage,
};
use tracing::{info, warn};
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

async fn chat_do_send_message<T: ChatMessageType>(
    chat: ReadOnlySignal<Option<ChatController<T>>>,
    message: T::M,
) -> Option<()> {
    let Some(controller) = chat.peek().clone() else {
        return None;
    };
    let sender = controller.sender();
    match sender.broadcast_message(message).await {
        Ok(_r) => Some(()),
        Err(e) => {
            warn!("Failed to send message: {}", e);
            None
        }
    }
}

pub fn chat_send_message_preview<T: ChatMessageType>(
    mm: ReadOnlySignal<Option<GlobalMatchmaker>>,
    message: T::M,
) -> Option<ReceivedMessage<T>> {
    let Some(mm) = mm.peek().clone() else {
        return None;
    };
    Some(ReceivedMessage {
        _sender_timestamp: datetime_now(),
        _received_timestamp: datetime_now(),
        _message_id: uuid::Uuid::new_v4(),
        from: mm.own_node_identity(),
        message: message.clone(),
    })
}

pub type ChatControllerSignal<T> = ReadOnlySignal<Option<ChatController<T>>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChatSignals<T: ChatMessageType> {
    pub chat: ChatControllerSignal<T>,
    pub presence: ReadOnlySignal<PresenceList<T>>,
    pub history: Signal<ChatHistory<T>>,
    pub send_user_message: Callback<T::M, Option<ReceivedMessage<T>>>,
}

fn use_chat_controller_signal<
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
    let r = chat.into();
    use_drop(move || {
        info!("use_drop: chat controller signals for T={}", std::any::type_name::<T>());
        let Some(chat) = chat.peek().clone() else {
            warn!("Chat is not initialized, so nothing to shutdown.");
            return;
        };
        n0_future::task::spawn(async move {
            if let Err(e) = chat.shutdown().await{
                warn!("Failed to shutdown chat {}: {}", std::any::type_name::<T>(), e);
            } else {
                info!("Chat {} shutdown successfully.", std::any::type_name::<T>());
            }
        });
    });
    r
}

fn use_chat_presence_signal<T: ChatMessageType>(
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

fn use_chat_history_signal<T: ChatMessageType>(
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

fn use_chat_send_message_callback<T: ChatMessageType>(
    chat: ChatControllerSignal<T>,
    mut history: Signal<ChatHistory<T>>,
) -> Callback<T::M, Option<ReceivedMessage<T>>> {
    let mm = use_context::<NetworkState>().global_mm;
    let coro = use_coroutine(move |mut n: UnboundedReceiver<T::M>| async move {
        while let Some(message) = n.next().await {
            chat_do_send_message(chat.into(), message).await;
        }
    });
    let on_user_message =
        Callback::new(move |message: <T as IChatRoomType>::M| {
            let m = chat_send_message_preview(mm.clone(),  message);
            if let Some(m) = &m {
                history.write().push(Ok(m.clone()));
                coro.send(m.message.clone());
            } else {
                    history
                    .write()
                    .push(Err("Failed to send message".to_string()));
                }
            m
        });
    on_user_message
}

pub fn use_chat_signals<
    T: ChatMessageType,
    Fut: 'static + Future<Output = Option<ChatController<T>>>,
>(
    get_chat_fn: Callback<GlobalMatchmaker, Fut>,
) -> ChatSignals<T> {
    info!("use_chat_signals<{}>", std::any::type_name::<T>());
    let chat = use_chat_controller_signal(get_chat_fn);
    let presence = use_chat_presence_signal(chat);
    let history = use_chat_history_signal(chat);
    let on_user_message = use_chat_send_message_callback(chat, history);
    ChatSignals { chat, presence, history, send_user_message: on_user_message }
}
pub fn use_global_chat_controller_signal(
) -> ChatSignals<GlobalChatMessageType> {
    info!("use_global_chat_controller_signal");
    use_chat_signals(Callback::new(move |mm: GlobalMatchmaker| async move {
        Some(mm.global_chat_controller().await?)
    }))
}

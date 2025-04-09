use std::{collections::VecDeque, future::Future};

use super::chat_traits::ChatMessageType;
use crate::network::NetworkState;
use dioxus::prelude::*;
use n0_future::StreamExt;
use protocol::{
    chat::{ChatController, IChatController, IChatReceiver, IChatSender},
    chat_presence::PresenceList,
    global_chat::GlobalChatMessageType,
    global_matchmaker::GlobalMatchmaker,
    user_identity::NodeIdentity,
    IChatRoomType, ReceivedMessage,
};
use tracing::{info, warn};

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

async fn chat_do_broadcast_send_message<T: ChatMessageType>(
    chat: ReadOnlySignal<Option<ChatController<T>>>,
    message: T::M,
) -> Option<ReceivedMessage<T>> {
    let Some(controller) = chat.peek().clone() else {
        return None;
    };
    let sender = controller.sender();
    match sender.broadcast_message(message).await {
        Ok(r) => Some(r),
        Err(e) => {
            warn!("Failed to send message: {}", e);
            None
        }
    }
}

async fn chat_do_send_direct_message<T: ChatMessageType>(
    chat: ReadOnlySignal<Option<ChatController<T>>>,
    to: NodeIdentity,
    message: T::M,
) -> Option<ReceivedMessage<T>> {
    let Some(controller) = chat.peek().clone() else {
        return None;
    };
    let sender = controller.sender();
    match sender.direct_message(to, message).await {
        Ok(r) => Some(r),
        Err(e) => {
            warn!("Failed to send message: {}", e);
            None
        }
    }
}

pub type ChatControllerSignal<T> = ReadOnlySignal<Option<ChatController<T>>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChatSignals<T: ChatMessageType> {
    pub chat: ChatControllerSignal<T>,
    pub presence: ReadOnlySignal<PresenceList<T>>,
    pub history: Signal<ChatHistory<T>>,
    pub send_broadcast_user_message: Callback<T::M>,
    pub send_direct_user_message: Callback<(NodeIdentity, T::M)>,
}

fn use_chat_controller_signal<
    T: ChatMessageType,
    Fut: 'static + Future<Output = Option<ChatController<T>>>,
>(
    shutdown_on_drop: bool,
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
    if shutdown_on_drop {
        use_drop(move || {
            info!(
                "use_drop: chat controller signals for T={}",
                std::any::type_name::<T>()
            );
            let Some(chat) = chat.peek().clone() else {
                warn!("Chat is not initialized, so nothing to shutdown.");
                return;
            };
            n0_future::task::spawn(async move {
                if let Err(e) = chat.shutdown().await {
                    warn!(
                        "Failed to shutdown chat {}: {}",
                        std::any::type_name::<T>(),
                        e
                    );
                } else {
                    info!(
                        "Chat {} shutdown successfully.",
                        std::any::type_name::<T>()
                    );
                }
            });
        });
    }
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

fn use_chat_send_broadcast_message_callback<T: ChatMessageType>(
    chat: ChatControllerSignal<T>,
    mut history: Signal<ChatHistory<T>>,
) -> Callback<T::M> {
    let coro =
        use_coroutine(move |mut n: UnboundedReceiver<T::M>| async move {
            while let Some(message) = n.next().await {
                match chat_do_broadcast_send_message(chat.into(), message).await
                {
                    Some(r) => {
                        history.write().push(Ok(r));
                    }
                    None => {
                        history
                            .write()
                            .push(Err("Failed to send message".to_string()));
                    }
                }
            }
        });
    let on_user_message =
        Callback::new(move |message: <T as IChatRoomType>::M| {
            coro.send(message.clone());
        });
    on_user_message
}

fn use_chat_send_direct_message_callback<T: ChatMessageType>(
    chat: ChatControllerSignal<T>,
    mut history: Signal<ChatHistory<T>>,
) -> Callback<(NodeIdentity, T::M)> {
    let coro = use_coroutine(
        move |mut n: UnboundedReceiver<(NodeIdentity, T::M)>| async move {
            while let Some((to, message)) = n.next().await {
                match chat_do_send_direct_message(chat.into(), to, message)
                    .await
                {
                    Some(r) => {
                        history.write().push(Ok(r));
                    }
                    None => {
                        history
                            .write()
                            .push(Err("Failed to send message".to_string()));
                    }
                }
            }
        },
    );
    let on_user_message = Callback::new(
        move |(to, message): (NodeIdentity, <T as IChatRoomType>::M)| {
            coro.send((to, message.clone()));
        },
    );
    on_user_message
}

pub fn use_chat_signals<
    T: ChatMessageType,
    Fut: 'static + Future<Output = Option<ChatController<T>>>,
>(
    shutdown_on_drop: bool,
    get_chat_fn: Callback<GlobalMatchmaker, Fut>,
) -> ChatSignals<T> {
    info!("use_chat_signals<{}>", std::any::type_name::<T>());
    let chat = use_chat_controller_signal(shutdown_on_drop, get_chat_fn);
    let presence = use_chat_presence_signal(chat);
    let history = use_chat_history_signal(chat);
    let send_broadcast_user_message =
        use_chat_send_broadcast_message_callback(chat, history);
    let send_direct_user_message =
        use_chat_send_direct_message_callback(chat, history);
    ChatSignals {
        chat,
        presence,
        history,
        send_broadcast_user_message,
        send_direct_user_message,
    }
}

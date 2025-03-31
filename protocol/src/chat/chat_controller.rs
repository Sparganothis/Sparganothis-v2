use std::{marker::PhantomData, sync::Arc};

use chrono::{DateTime, Utc};
use futures::{FutureExt, StreamExt};
use iroh::NodeId;
use n0_future::task::{spawn, AbortOnDropHandle};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{
    _const::{CONNECT_TIMEOUT, PRESENCE_INTERVAL}, chat_presence::ChatPresence, chat_ticket::ChatTicket, datetime_now, signed_message::{IChatRoomType, MessageSigner, SignedMessage}, sleep::SleepManager, user_identity::NodeIdentity, ReceivedMessage
};

#[derive(Clone, Debug)]
pub struct ChatController<T: IChatRoomType> {
    ticket: ChatTicket,
    inner: Arc<dyn IChatRoomRaw>,
    presence: ChatPresence<T>,
    _p: PhantomData<T>,
    _dispatch_task: Arc<AbortOnDropHandle<anyhow::Result<()>>>,
    _presence_task: Arc<AbortOnDropHandle<anyhow::Result<()>>>,
    sender: ChatSender<T>,
    receiver: ChatReceiver<T>,
    _controller_id: uuid::Uuid,
}

impl<T: IChatRoomType> PartialEq for ChatController<T> {
    fn eq(&self, other: &Self) -> bool {
        self._controller_id == other._controller_id
    }
}

async fn _dispatch_inner_loop<T: IChatRoomType>(m: crate::WireMessage<ChatMessage<T>>, msg_sender: &mut async_broadcast::Sender<ReceivedMessage<T>>, _presence: ChatPresence<T>, _sender: ChatSender<T>) -> anyhow::Result<()> {
    match m.message {
        ChatMessage::Message(message) => {
            msg_sender
                .broadcast(ReceivedMessage {
                    _sender_timestamp: m._timestamp,
                    _received_timestamp: datetime_now(),
                    _message_id: m._message_id,
                    from: m.from,
                    message,
                })
                .await?;
        }
        ChatMessage::Presence(presence) => {
            let was_added = _presence
                .add_presence(&m.from, &presence)
                .await;
            if was_added {
                _sender.direct_presence(m.from).await?;
            }
            let ping_sender_ts = m._timestamp;
            _sender.direct_pong(m.from, ping_sender_ts).await?;
        }
        ChatMessage::Pong { ping_sender_ts } => {
            let now = datetime_now();
            let dt = now
                .signed_duration_since(ping_sender_ts)
                .to_std()
                .ok();
            if let Some(dt) = dt {
                let rtt = dt.as_micros() as f64 / 1000.0;
                if rtt > 0.0 && rtt < 65000.0 {
                    let rtt = rtt as u16 + 1;
                    _presence.update_ping(&m.from, rtt).await;
                }
            }
        }
    }
    Ok(())
}

impl<T: IChatRoomType> ChatController<T> {
    pub(crate) fn new(
        ticket: ChatTicket,
        inner: Arc<dyn IChatRoomRaw>,
        message_signer: MessageSigner,
        sleep_manager: SleepManager,
    ) -> Self {
        let presence = ChatPresence::new();
        let sender = ChatSender {
            inner: inner.clone(),
            message_signer: message_signer.clone(),
            current_presence: Arc::new(Mutex::new(None)),
            chatroom_presence: presence.clone(),
            _p: PhantomData,
        };
        let (mut msg_sender, mut msg_receiver) = async_broadcast::broadcast(16);
        msg_sender.set_overflow(true);
        msg_receiver.set_overflow(true);

        let msg_receiver = Arc::new(Mutex::new(msg_receiver));
        let receiver = ChatReceiver {
            msg_receiver: msg_receiver.clone(),
            _p: PhantomData,
        };
        let inner2 = inner.clone();
        let _presence = presence.clone();
        let _sender = sender.clone();
        let _sleep_manager = sleep_manager.clone();
        let _dispatch_task =async move {
            let mut errors = 0;
            loop {
                let Ok(Some(message)) = inner2.next_message().await else {
                    errors += 1;
                    if errors > 10 {
                        warn!("_dispatch_task: Chat room closed");
                        anyhow::bail!("_dispatch_task: Chat room closed");
                    }
                    _sleep_manager
                        .sleep(std::time::Duration::from_millis(8))
                        .await;
                    continue;
                };
                errors = 0;
                let msg = SignedMessage::verify_and_decode::<ChatMessage<T>>(
                    &message,
                );
                match msg {
                    Ok(m) => {
                        if let Err(e) = _dispatch_inner_loop::<T>(m, &mut msg_sender, _presence.clone(), _sender.clone()).await {
                            warn!("_dispatch_task: Error dispatching message: {:?}", e);
                        }
                    },
                    Err(e) => {
                        warn!(
                            "_dispatch_task: Error verifying message: {:?}",
                            e
                        );
                    }
                }
            }
        };
        let _dispatch_task = AbortOnDropHandle::new(spawn(async move {
            let r = _dispatch_task.await;
            warn!("_dispatch_task: Chat room closed: {r:#?}");
            r
        }));

        let _sleep_manager = sleep_manager.clone();
        let _sender = sender.clone();
        let _presence_task =async move {
            loop {
                let _ = _sleep_manager.sleep(PRESENCE_INTERVAL).await;
                if let Err(e) = _sender.broadcast_presence().await {
                    warn!(
                        "_presence_task: Error broadcasting presence: {:?}",
                        e
                    );
                }
            }
        };
        let _presence_task = AbortOnDropHandle::new(spawn(async move {
            let r = _presence_task.await;
            warn!("_presence_task: Chat room closed: {r:#?}");
            r
        }));

        let controller = Self {
            _controller_id: uuid::Uuid::new_v4(),
            inner,
            presence,
            _p: PhantomData,
            _dispatch_task: Arc::new(_dispatch_task),
            _presence_task: Arc::new(_presence_task),
            sender,
            receiver,
            ticket,
        };
        controller
    }

}

#[async_trait::async_trait]
impl<T: IChatRoomType> IChatController<T> for ChatController<T> {
    fn sender(&self) -> ChatSender<T> {
        self.sender.clone()
    }
    async fn receiver(&self) -> ChatReceiver<T> {
        let new_receiver = {
            let mut m = self.receiver.msg_receiver.lock().await;
            let mut new_receiver = m.clone();
            let m2 = &mut *m;
            std::mem::swap(m2, &mut new_receiver);
            new_receiver
        };
        ChatReceiver {
            msg_receiver: Arc::new(Mutex::new(new_receiver)),
            _p: PhantomData,
        }
    }
    async fn shutdown(&self) -> anyhow::Result<()> {
        self.inner.shutdown().await
    }
    fn chat_presence(&self) -> ChatPresence<T> {
        self.presence.clone()
    }
    async fn wait_joined(&self) -> anyhow::Result<()> {
        let mut bootstrap = self.ticket.bootstrap.clone();
        if bootstrap.is_empty() {
            return Ok(());
        }
        let mut x = 0;
        let p = self.chat_presence();
        while !bootstrap.is_empty() && x <= 3 {
            let presence_list = p.get_presence_list().await.into_iter().map(|p| *p.2.node_id()).collect::<Vec<_>>();
            info!("wait_until_joined: found {:?}/{:?}", presence_list.len(), bootstrap.len());
            if presence_list.len() >= 3 {
                break;
            }
            for k in presence_list {
                if bootstrap.remove(&k) {
                    x += 1;
                }
            }
            if bootstrap.is_empty() || x >= 3 {
                break;
            }
           let _ =  n0_future::time::timeout(CONNECT_TIMEOUT/20, p.notified()).await;
            x += 1;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ChatMessage<T: IChatRoomType> {
    Message(T::M),
    Presence(Option<T::P>),
    Pong { ping_sender_ts: DateTime<Utc> },
}

#[async_trait::async_trait]
pub trait IChatController<T: IChatRoomType>:
    Send + Sync + 'static + std::fmt::Debug
{
    fn sender(&self) -> ChatSender<T>;
    async fn receiver(&self) -> ChatReceiver<T>;
    async fn shutdown(&self) -> anyhow::Result<()>;
    fn chat_presence(&self) -> ChatPresence<T>;
    async fn wait_joined(&self) -> anyhow::Result<()>;
}

#[derive(Clone, Debug)]
pub struct ChatSender<T: IChatRoomType> {
    inner: Arc<dyn IChatRoomRaw>,
    message_signer: MessageSigner,
    current_presence: Arc<Mutex<Option<T::P>>>,
    chatroom_presence: ChatPresence<T>,
    _p: PhantomData<T>,
}

#[async_trait::async_trait]
impl<T: IChatRoomType> IChatSender<T> for ChatSender<T> {
    async fn broadcast_message(&self, message: T::M) -> anyhow::Result<()> {
        let message = ChatMessage::<T>::Message(message);
        let message = self.message_signer.sign_and_encode(message)?;
        self.inner.broadcast_message(message).await
    }
    async fn direct_message(
        &self,
        to: NodeIdentity,
        message: T::M,
    ) -> anyhow::Result<()> {
        let message = ChatMessage::<T>::Message(message);
        let message = self.message_signer.sign_and_encode(message)?;
        self.inner.direct_message(to, message).await
    }
    async fn join_peers(&self, peers: Vec<NodeId>) -> anyhow::Result<()> {
        self.inner.join_peers(peers).await
    }
    async fn set_presence(&self, presence: &T::P) {
        self.current_presence.lock().await.replace(presence.clone());
        if let Err(e) = self.broadcast_presence().await {
            warn!("Error broadcasting presence: {:?}", e);
        }
    }
}

impl<T: IChatRoomType> ChatSender<T> {
    async fn broadcast_presence(&self) -> anyhow::Result<()> {
        let presence = { self.current_presence.lock().await.clone() };
        self.chatroom_presence
            .add_presence(&self.message_signer.node_identity, &presence)
            .await;
        let presence = ChatMessage::<T>::Presence(presence);
        let presence = self.message_signer.sign_and_encode(presence)?;
        self.inner.broadcast_message(presence).await
    }
    async fn direct_presence(&self, to: NodeIdentity) -> anyhow::Result<()> {
        let presence = { self.current_presence.lock().await.clone() };

        let presence = ChatMessage::<T>::Presence(presence);
        let presence = self.message_signer.sign_and_encode(presence)?;
        self.inner.direct_message(to, presence).await
    }
    async fn direct_pong(
        &self,
        to: NodeIdentity,
        ping_sender_ts: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let pong = ChatMessage::<T>::Pong { ping_sender_ts };
        let pong = self.message_signer.sign_and_encode(pong)?;
        self.inner.direct_message(to, pong).await
    }
}

#[async_trait::async_trait]
pub trait IChatSender<T: IChatRoomType>:
    Send + Sync + 'static + std::fmt::Debug
{
    async fn broadcast_message(&self, message: T::M) -> anyhow::Result<()>;
    async fn direct_message(
        &self,
        to: NodeIdentity,
        message: T::M,
    ) -> anyhow::Result<()>;
    async fn join_peers(&self, peers: Vec<NodeId>) -> anyhow::Result<()>;
    async fn set_presence(&self, presence: &T::P);
}

#[derive(Clone, Debug)]
pub struct ChatReceiver<T: IChatRoomType> {
    msg_receiver: Arc<Mutex<async_broadcast::Receiver<ReceivedMessage<T>>>>,
    _p: PhantomData<T>,
}

#[async_trait::async_trait]
impl<T: IChatRoomType> IChatReceiver<T> for ChatReceiver<T> {
    async fn next_message(&self) -> Option<ReceivedMessage<T>> {
        Some(self.msg_receiver.lock().await.next().fuse().await?)
    }
}

#[async_trait::async_trait]
pub trait IChatReceiver<T: IChatRoomType>:
    Send + Sync + 'static + std::fmt::Debug
{
    async fn next_message(&self) -> Option<ReceivedMessage<T>>;
}

#[async_trait::async_trait]
pub trait IChatRoomRaw: Send + Sync + 'static + std::fmt::Debug {
    async fn broadcast_message(&self, message: Vec<u8>) -> anyhow::Result<()>;
    async fn direct_message(
        &self,
        to: NodeIdentity,
        message: Vec<u8>,
    ) -> anyhow::Result<()>;
    async fn next_message(&self) -> anyhow::Result<Option<Arc<Vec<u8>>>>;
    async fn join_peers(&self, peers: Vec<NodeId>) -> anyhow::Result<()>;
    async fn shutdown(&self) -> anyhow::Result<()>;
}

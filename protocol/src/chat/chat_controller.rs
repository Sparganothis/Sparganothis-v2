use std::{marker::PhantomData, sync::Arc};

use chrono::{DateTime, Utc};
use futures::{FutureExt, StreamExt};
use iroh::NodeId;
use matchbox_socket::{async_trait, PeerState};
use n0_future::task::{spawn, AbortOnDropHandle};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{
    _const::PRESENCE_INTERVAL,
    chat_presence::ChatPresence,
    datetime_now,
    signed_message::{IChatRoomType, MessageSigner, SignedMessage},
    sleep::SleepManager,
    user_identity::NodeIdentity,
    ReceivedMessage,
    _matchbox_signal::PeerTracker,
};

#[derive(Clone, Debug)]
pub struct ChatController<T: IChatRoomType> {
    inner: Arc<dyn IChatRoomRaw>,
    presence: ChatPresence<T>,
    _p: PhantomData<T>,
    _dispatch_task: Arc<AbortOnDropHandle<anyhow::Result<()>>>,
    _events_task: Arc<AbortOnDropHandle<anyhow::Result<()>>>,
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

impl<T: IChatRoomType> ChatController<T> {
    pub(crate) async fn peer_tracker(&self) -> PeerTracker {
        self.inner.peer_tracker().await
    }
    pub(crate) fn new(
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
        let _dispatch_task = AbortOnDropHandle::new(spawn(async move {
            while let Some((peer, message)) = inner2.next_message().await {
                let msg = SignedMessage::verify_and_decode::<ChatMessage<T>>(
                    &message,
                );
                match msg {
                    Ok(m) => {
                        assert_eq!(m.from, peer);
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
                                _presence
                                    .add_presence(&m.from, &presence)
                                    .await;
                                let ping_sender_ts = m._timestamp;
                                _sender
                                    .direct_pong(m.from, ping_sender_ts)
                                    .await?;
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
                                        _presence
                                            .update_ping(&m.from, rtt)
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!(
                            "_dispatch_task: Error verifying message: {:?}",
                            e
                        );
                    }
                }
            }
            warn!("_dispatch_task: Chat room closed");
            anyhow::bail!("_dispatch_task: Chat room closed");
        }));

        let inner2 = inner.clone();
        let _sender = sender.clone();
        let _presence = presence.clone();
        let _events_task = AbortOnDropHandle::new(spawn(async move {
            let mut err = 0;
            loop {
                if let Some((peer, event)) = inner2.next_peer_event().await {
                    match event {
                        PeerState::Connected => {
                            let peer_tracker = inner2.peer_tracker().await;
                            peer_tracker.confirm_peer(peer).await;
                            _sender.direct_presence(peer).await?;
                            info!(
                                "_events_task: \n Peer connected: {:?}",
                                peer.matchbox_id()
                            );
                        }
                        PeerState::Disconnected => {
                            // let peer_tracker = inner2.peer_tracker().await;
                            // peer_tracker.drop_peers(vec![peer]).await;
                            _presence.remove_presence(&peer).await;
                            info!(
                                "_events_task: \n Peer disconnected: {:?}",
                                peer.matchbox_id()
                            );
                        }
                    }
                    err = 0;
                } else {
                    err += 1;
                    if err > 10 {
                        warn!("_events_task: Events task closed");
                        anyhow::bail!("_events_task: Events task closed");
                    }
                }
            }
        }));
        let _sleep_manager = sleep_manager.clone();
        let _sender = sender.clone();
        let _presence_task = AbortOnDropHandle::new(spawn(async move {
            loop {
                let _ = _sleep_manager.sleep(PRESENCE_INTERVAL).await;
                if let Err(e) = _sender.broadcast_presence().await {
                    warn!(
                        "_presence_task: Error broadcasting presence: {:?}",
                        e
                    );
                }
            }
        }));

        Self {
            _controller_id: uuid::Uuid::new_v4(),
            inner,
            presence,
            _p: PhantomData,
            _dispatch_task: Arc::new(_dispatch_task),
            _events_task: Arc::new(_events_task),
            _presence_task: Arc::new(_presence_task),
            sender,
            receiver,
        }
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
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ChatMessage<T: IChatRoomType> {
    Message(T::M),
    Presence(T::P),
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
        let Some(presence) = presence else {
            return Ok(());
        };
        self.chatroom_presence
            .add_presence(&self.message_signer.node_identity, &presence)
            .await;
        let presence = ChatMessage::<T>::Presence(presence);
        let presence = self.message_signer.sign_and_encode(presence)?;
        self.inner.broadcast_message(presence).await
    }
    async fn direct_presence(&self, to: NodeIdentity) -> anyhow::Result<()> {
        let presence = { self.current_presence.lock().await.clone() };
        let Some(presence) = presence else {
            return Ok(());
        };
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
    async fn next_message(&self) -> Option<(NodeIdentity, Vec<u8>)>;
    async fn next_peer_event(&self) -> Option<(NodeIdentity, PeerState)>;
    async fn join_peers(&self, peers: Vec<NodeId>) -> anyhow::Result<()>;
    async fn shutdown(&self) -> anyhow::Result<()>;
    async fn peer_tracker(&self) -> PeerTracker;
}

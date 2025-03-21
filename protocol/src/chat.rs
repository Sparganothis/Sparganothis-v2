use std::{collections::BTreeSet, sync::Arc};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
pub use iroh::NodeId;
use iroh::{PublicKey, SecretKey};
use iroh_base::Signature;
use iroh_gossip::net::{Gossip, GossipEvent, GossipSender};
pub use iroh_gossip::proto::TopicId;
use n0_future::{
    task::{self, AbortOnDropHandle},
    time::{Duration, SystemTime},
    StreamExt,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;
use tracing::{error, info, warn};

use crate::{
    _const::CONNECT_TIMEOUT,
    chat_presence::ChatPresence,
    user_identity::{NodeIdentity, UserIdentitySecrets},
};
use crate::{_const::PRESENCE_INTERVAL, sleep::SleepManager};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatTicket {
    pub topic_id: TopicId,
    pub bootstrap: BTreeSet<NodeId>,
}

impl ChatTicket {
    pub fn new_str_bs(topic_id: &str, bs: BTreeSet<NodeId>) -> Self {
        let mut topic_bytes = [0; 32];
        let topic_str = topic_id.as_bytes().to_vec();
        assert!(topic_str.len() <= 30);
        topic_bytes[..topic_str.len()].copy_from_slice(&topic_str);
        Self {
            topic_id: TopicId::from_bytes(topic_bytes),
            bootstrap: bs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatSender {
    user_secrets: Arc<UserIdentitySecrets>,
    node_secret_key: Arc<SecretKey>,
    node_identity: Arc<NodeIdentity>,
    sender: GossipSender,
    _trigger_presence: Arc<Notify>,
    _presence_task: Arc<AbortOnDropHandle<()>>,
}

impl ChatSender {
    pub async fn send(&self, text: String) -> Result<()> {
        let message = ChatMessage::Message { text };
        let signed_message = SignedMessage::sign_and_encode(
            &self.node_secret_key.as_ref(),
            &self.user_secrets.as_ref().secret_key(),
            message,
            self.node_identity.clone(),
        )?;
        self.sender.broadcast(signed_message.into()).await?;
        Ok(())
    }
    pub async fn join_peers(&self, peers: Vec<NodeId>) -> Result<()> {
        let me = self.node_identity.node_id().clone();
        let peers: Vec<PublicKey> =
            peers.into_iter().filter(|id| id != &me).collect();
        if peers.is_empty() {
            return Ok(());
        }
        self.sender.join_peers(peers).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkEvent {
    Message { event: ReceivedMessage },

    NetworkChange { event: NetworkChangeEvent },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkChangeEvent {
    Joined { neighbors: Vec<NodeId> },

    NeighborUp { node_id: NodeId },
    NeighborDown { node_id: NodeId },
    Lagged,
}

impl TryFrom<iroh_gossip::net::Event> for NetworkEvent {
    type Error = anyhow::Error;
    fn try_from(event: iroh_gossip::net::Event) -> Result<Self, Self::Error> {
        let converted = match event {
            iroh_gossip::net::Event::Gossip(event) => match event {
                GossipEvent::Joined(neighbors) => Self::NetworkChange {
                    event: NetworkChangeEvent::Joined { neighbors },
                },
                GossipEvent::NeighborUp(node_id) => Self::NetworkChange {
                    event: NetworkChangeEvent::NeighborUp { node_id },
                },
                GossipEvent::NeighborDown(node_id) => Self::NetworkChange {
                    event: NetworkChangeEvent::NeighborDown { node_id },
                },
                GossipEvent::Received(message) => {
                    let message =
                        SignedMessage::verify_and_decode(&message.content)
                            .context(
                                "failed to parse and verify signed message",
                            )?;
                    Self::Message { event: message }
                }
            },
            iroh_gossip::net::Event::Lagged => {
                error!("Lagged! Channel will be closed!");
                Self::NetworkChange {
                    event: NetworkChangeEvent::Lagged,
                }
            }
        };
        Ok(converted)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedMessage {
    node_pubkey: PublicKey,
    user_pubkey: PublicKey,
    data: Vec<u8>,
    node_signature: Signature,
    user_signature: Signature,
}

impl SignedMessage {
    pub fn verify_and_decode(bytes: &[u8]) -> Result<ReceivedMessage> {
        let signed_message: Self = postcard::from_bytes(bytes)?;
        let message: WireMessage = postcard::from_bytes(&signed_message.data)?;
        let WireMessage::VO {
            timestamp,
            message,
            from,
        } = message;
        if from.user_id() != &signed_message.user_pubkey {
            return Err(anyhow::anyhow!("user id mismatch"));
        }
        if from.node_id() != &signed_message.node_pubkey {
            return Err(anyhow::anyhow!("node id mismatch"));
        }

        signed_message
            .node_pubkey
            .verify(&signed_message.data, &signed_message.node_signature)?;
        signed_message
            .user_pubkey
            .verify(&signed_message.data, &signed_message.user_signature)?;

        Ok(ReceivedMessage {
            from,
            timestamp,
            message,
        })
    }

    pub fn sign_and_encode(
        node_secret_key: &SecretKey,
        user_secret_key: &SecretKey,
        message: ChatMessage,
        from: Arc<NodeIdentity>,
    ) -> Result<Vec<u8>> {
        let timestamp = timestamp_now();
        let wire_message = WireMessage::VO {
            timestamp,
            message,
            from: from.as_ref().clone(),
        };
        let data = postcard::to_stdvec(&wire_message)?;
        let node_signature = node_secret_key.sign(&data);
        let user_signature = user_secret_key.sign(&data);
        let signed_message = Self {
            node_pubkey: node_secret_key.public(),
            user_pubkey: user_secret_key.public(),
            data,
            node_signature,
            user_signature,
        };
        let encoded = postcard::to_stdvec(&signed_message)?;
        Ok(encoded)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WireMessage {
    VO {
        timestamp: DateTime<Utc>,
        message: ChatMessage,
        from: NodeIdentity,
    },
}

pub fn timestamp_now() -> DateTime<Utc> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;
    DateTime::<Utc>::from_timestamp_micros(timestamp as i64).unwrap()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChatMessage {
    Presence {},
    Message { text: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ReceivedMessage {
    pub timestamp: DateTime<Utc>,
    pub from: NodeIdentity,
    pub message: ChatMessage,
}

pub type ChatEventStream = std::pin::Pin<
    Box<
        (dyn tokio_stream::Stream<
            Item = Result<crate::chat::NetworkEvent, anyhow::Error>,
        > + std::marker::Send
             + 'static),
    >,
>;

pub async fn join_chat(
    gossip: Gossip,
    node_secret_key: Arc<SecretKey>,
    ticket: &ChatTicket,
    user_secrets: Arc<UserIdentitySecrets>,
    node_identity: Arc<NodeIdentity>,
    sleep_: SleepManager,
) -> Result<ChatController> {
    let topic_id = ticket.topic_id;
    let bootstrap: Vec<PublicKey> = ticket
        .bootstrap
        .iter()
        .filter(|id| id != &node_identity.node_id())
        .cloned()
        .collect();
    // let bootstrap_count = bootstrap.len();
    info!("joining {topic_id} : {bootstrap:#?}");
    // let gossip_topic = if bootstrap_count == 0 {
        // info!("try subscribe with zero nodes");
        let gossip_topic =gossip.subscribe(topic_id, bootstrap)?;
    // } else {
        // info!("try subscribe with {bootstrap_count} nodes");
        // n0_future::time::timeout(
            // CONNECT_TIMEOUT,
            // gossip.subscribe_and_join(topic_id, bootstrap),
        // )
        // .await
        // .context("join chat")?
        // .context("join chat")?
    // };
    let (sender, receiver) = gossip_topic.split();

    let trigger_presence = Arc::new(Notify::new());
    let presence = ChatPresence::new();

    // We spawn a task that occasionally sens a Presence message with our nickname.
    // This allows to track which peers are online currently.
    let presence_task = AbortOnDropHandle::new(task::spawn({
        let node_secret_key = node_secret_key.clone();
        let sender = sender.clone();
        let trigger_presence = trigger_presence.clone();
        let sleep_ = sleep_.clone();
        let user_secrets = user_secrets.clone();
        let node_identity = node_identity.clone();
        let presence = presence.clone();
        async move {
            // trigger_presence.notified().await;
            loop {
                let message = ChatMessage::Presence {};
                let signed_message = SignedMessage::sign_and_encode(
                    &node_secret_key,
                    &user_secrets.as_ref().secret_key(),
                    message,
                    node_identity.clone(),
                )
                .expect("failed to encode message");
                if let Err(err) = sender.broadcast(signed_message.into()).await
                {
                    tracing::warn!("presence task failed to broadcast: {err}");
                    break;
                }
                presence.add_presence(&node_identity).await;
                let wait = PRESENCE_INTERVAL
                    + Duration::from_secs(rand::thread_rng().gen_range(0..3));
                n0_future::future::race(
                    sleep_.sleep(wait),
                    trigger_presence.notified(),
                )
                .await;
            }
        }
    }));

    // We create a stream of events, coming from the gossip topic event receiver.
    // We'll want to map the events to our own event type, which includes parsing
    // the messages and verifying the signatures, and trigger presence
    // once the swarm is joined initially.
    let receiver = n0_future::stream::try_unfold(receiver, {
        let trigger_presence = trigger_presence.clone();
        move |mut receiver| {
            let trigger_presence = trigger_presence.clone();
            async move {
                // trigger_presence.notify_waiters();
                // trigger_presence.notify_one();
                loop {
                    // Store if we were joined before the next event comes in.
                    let was_joined = receiver.is_joined();

                    // Fetch the next event.
                    let Some(event) = receiver.try_next().await? else {
                        return Ok(None);
                    };
                    // Convert into our event type. this fails if we receive a message
                    // that cannot be decoced into our event type. If that is the case,
                    // we just keep and log the error.
                    let event: NetworkEvent = match event.try_into() {
                        Ok(event) => event,
                        Err(err) => {
                            warn!("BOOTSTRAP: received invalid message: {err}");
                            continue;
                        }
                    };
                    // If we just joined, trigger sending our presence message.
                    if !was_joined && receiver.is_joined() {
                        trigger_presence.notify_waiters()
                    };

                    break Ok(Some((event, receiver)));
                }
            }
        }
    });

    let sender = ChatSender {
        node_secret_key,
        sender,
        _trigger_presence: trigger_presence,
        _presence_task: Arc::new(presence_task),
        user_secrets,
        node_identity,
    };
    Ok(ChatControllerRaw {
        sender,
        receiver: Box::pin(receiver),
        sleep_manager: sleep_,
        presence,
    }
    .into())
}

#[derive(Clone)]
pub struct ChatController {
    inner: Arc<ChatControllerInner>,
    chat_presence: ChatPresence,
}

impl ChatController {
    pub fn sender(&self) -> ChatSender {
        self.inner.sender.clone()
    }
    pub fn receiver(&self) -> ChatEventReceiver {
        self.inner.receiver.activate_cloned()
    }
    pub async fn shutdown(self) -> Result<()> {
        info!("ChatController shutdown");
        self.inner.receiver.close();

        Ok(())
    }
    pub fn chat_presence(&self) -> ChatPresence {
        self.chat_presence.clone()
    }
}

#[derive(Clone)]
pub struct ChatEventStreamError(Arc<anyhow::Error>);

impl ChatEventStreamError {
    pub fn original_err(&self) -> &anyhow::Error {
        &self.0
    }
}

impl std::fmt::Debug for ChatEventStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChatEventStreamError({:#?})", self.0)
    }
}

impl std::fmt::Display for ChatEventStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChatEventStreamError({})", self.0)
    }
}

impl std::error::Error for ChatEventStreamError {}

pub type ChatEventReceiver =
    async_broadcast::Receiver<Result<NetworkEvent, ChatEventStreamError>>;
pub type ChatEventReceiverInactive = async_broadcast::InactiveReceiver<
    Result<NetworkEvent, ChatEventStreamError>,
>;
struct ChatControllerInner {
    sender: ChatSender,
    receiver: ChatEventReceiverInactive,
    _recv_broadcast_task: AbortOnDropHandle<()>,
}

struct ChatControllerRaw {
    pub sender: ChatSender,
    pub receiver: ChatEventStream,
    pub sleep_manager: SleepManager,
    pub presence: ChatPresence,
}

impl Into<ChatController> for ChatControllerRaw {
    fn into(self) -> ChatController {
        let ChatControllerRaw {
            sender,
            mut receiver,
            sleep_manager,
            presence,
        } = self;
        let (mut b_sender, b_recv) = async_broadcast::broadcast(128);
        b_sender.set_overflow(true);
        let presence_ = presence.clone();
        let sleep_ = sleep_manager.clone();
        let task = AbortOnDropHandle::new(task::spawn(async move {
            while let Some(event) = receiver.next().await {
                sleep_.wake_up();
                let event =
                    event.map_err(|e| ChatEventStreamError(Arc::new(e)));
                let _r = b_sender.broadcast(event.clone()).await;
                if let Err(broadcast_err) = _r {
                    error!("chat controller raw receiver stream error: {broadcast_err}");
                    break;
                }
                if let Ok(NetworkEvent::Message {
                    event:
                        ReceivedMessage {
                            message: ChatMessage::Presence {},
                            from,
                            ..
                        },
                }) = &event
                {
                    presence_.add_presence(&from).await;
                }
            }
            warn!("XXX: chat controller raw receiver stream closed.");
        }));
        ChatController {
            inner: Arc::new(ChatControllerInner {
                sender,
                receiver: b_recv.deactivate(),
                _recv_broadcast_task: task,
            }),
            chat_presence: presence,
        }
    }
}

use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
pub use iroh::NodeId;
use iroh::{PublicKey, SecretKey};
use iroh_base::{ticket::Ticket, Signature};
use iroh_gossip::net::{Gossip, GossipEvent, GossipSender};
pub use iroh_gossip::proto::TopicId;
use n0_future::{
    task::{self, AbortOnDropHandle},
    time::{Duration, SystemTime},
    StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;
use tracing::{debug, info, warn};

pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(35);

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
    pub fn new_random() -> Self {
        let topic_id = TopicId::from_bytes(rand::random());
        Self::new(topic_id)
    }

    pub fn new(topic_id: TopicId) -> Self {
        Self {
            topic_id,
            bootstrap: Default::default(),
        }
    }
    pub fn deserialize(input: &str) -> Result<Self> {
        <Self as Ticket>::deserialize(input).map_err(Into::into)
    }
    pub fn serialize(&self) -> String {
        <Self as Ticket>::serialize(self)
    }
}

impl Ticket for ChatTicket {
    const KIND: &'static str = "chat";

    fn to_bytes(&self) -> Vec<u8> {
        postcard::to_stdvec(&self).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, iroh_base::ticket::Error> {
        let ticket = postcard::from_bytes(bytes)?;
        Ok(ticket)
    }
}

#[derive(Debug, Clone)]
pub struct ChatSender {
    nickname: Arc<Mutex<String>>,
    secret_key: SecretKey,
    sender: GossipSender,
    trigger_presence: Arc<Notify>,
    _presence_task: Arc<AbortOnDropHandle<()>>,
}

impl ChatSender {
    pub async fn send(&self, text: String) -> Result<()> {
        let nickname = self.nickname.lock().expect("poisened").clone();
        let message = Message::Message { text, nickname };
        let signed_message = SignedMessage::sign_and_encode(&self.secret_key, message)?;
        self.sender.broadcast(signed_message.into()).await?;
        Ok(())
    }

    pub fn set_nickname(&self, name: String) {
        *self.nickname.lock().expect("poisened") = name;
        self.trigger_presence.notify_waiters();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatEvent {
    #[serde(rename_all = "camelCase")]
    Joined {
        neighbors: Vec<NodeId>,
    },
    #[serde(rename_all = "camelCase")]
    MessageReceived {
        from: NodeId,
        text: String,
        nickname: String,
        sent_timestamp: u64,
    },
    #[serde(rename_all = "camelCase")]
    Presence {
        from: NodeId,
        nickname: String,
        sent_timestamp: u64,
    },
    #[serde(rename_all = "camelCase")]
    NeighborUp {
        node_id: NodeId,
    },
    #[serde(rename_all = "camelCase")]
    NeighborDown {
        node_id: NodeId,
    },
    Lagged,
}

impl TryFrom<iroh_gossip::net::Event> for ChatEvent {
    type Error = anyhow::Error;
    fn try_from(event: iroh_gossip::net::Event) -> Result<Self, Self::Error> {
        let converted = match event {
            iroh_gossip::net::Event::Gossip(event) => match event {
                GossipEvent::Joined(neighbors) => Self::Joined { neighbors },
                GossipEvent::NeighborUp(node_id) => Self::NeighborUp { node_id },
                GossipEvent::NeighborDown(node_id) => Self::NeighborDown { node_id },
                GossipEvent::Received(message) => {
                    let message = SignedMessage::verify_and_decode(&message.content)
                        .context("failed to parse and verify signed message")?;
                    match message.message {
                        Message::Presence { nickname } => Self::Presence {
                            from: message.from,
                            nickname,
                            sent_timestamp: message.timestamp,
                        },
                        Message::Message { text, nickname } => Self::MessageReceived {
                            from: message.from,
                            text,
                            nickname,
                            sent_timestamp: message.timestamp,
                        },
                    }
                }
            },
            iroh_gossip::net::Event::Lagged => Self::Lagged,
        };
        Ok(converted)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedMessage {
    from: PublicKey,
    data: Vec<u8>,
    signature: Signature,
}

impl SignedMessage {
    pub fn verify_and_decode(bytes: &[u8]) -> Result<ReceivedMessage> {
        let signed_message: Self = postcard::from_bytes(bytes)?;
        let key: PublicKey = signed_message.from;
        key.verify(&signed_message.data, &signed_message.signature)?;
        let message: WireMessage = postcard::from_bytes(&signed_message.data)?;
        let WireMessage::VO { timestamp, message } = message;
        Ok(ReceivedMessage {
            from: signed_message.from,
            timestamp,
            message,
        })
    }

    pub fn sign_and_encode(secret_key: &SecretKey, message: Message) -> Result<Vec<u8>> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        let wire_message = WireMessage::VO { timestamp, message };
        let data = postcard::to_stdvec(&wire_message)?;
        let signature = secret_key.sign(&data);
        let from: PublicKey = secret_key.public();
        let signed_message = Self {
            from,
            data,
            signature,
        };
        let encoded = postcard::to_stdvec(&signed_message)?;
        Ok(encoded)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WireMessage {
    VO { timestamp: u64, message: Message },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Presence { nickname: String },
    Message { text: String, nickname: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceivedMessage {
    timestamp: u64,
    from: NodeId,
    message: Message,
}

pub type ChatEventStream = std::pin::Pin<
    Box<
        (dyn tokio_stream::Stream<Item = Result<crate::chat::ChatEvent, anyhow::Error>>
             + std::marker::Send
             + 'static),
    >,
>;

pub fn join_chat(
    gossip: Gossip,
    secret_key: SecretKey,
    nickname: String,
    ticket: &ChatTicket,
) -> Result<(ChatSender, ChatEventStream)> {
    let topic_id = ticket.topic_id;
    let bootstrap = ticket.bootstrap.iter().cloned().collect();
    info!(?bootstrap, "joining {topic_id}");
    let gossip_topic = gossip.subscribe(topic_id, bootstrap)?;
    let (sender, receiver) = gossip_topic.split();

    let nickname = Arc::new(Mutex::new(nickname));
    let trigger_presence = Arc::new(Notify::new());

    // We spawn a task that occasionally sens a Presence message with our nickname.
    // This allows to track which peers are online currently.
    let presence_task = AbortOnDropHandle::new(task::spawn({
        let secret_key = secret_key.clone();
        let sender = sender.clone();
        let trigger_presence = trigger_presence.clone();
        let nickname = nickname.clone();

        async move {
            loop {
                let nickname = nickname.lock().expect("poisened").clone();
                let message = Message::Presence { nickname };
                debug!("send presence {message:?}");
                let signed_message = SignedMessage::sign_and_encode(&secret_key, message)
                    .expect("failed to encode message");
                if let Err(err) = sender.broadcast(signed_message.into()).await {
                    tracing::warn!("presence task failed to broadcast: {err}");
                    break;
                }
                n0_future::future::race(
                    n0_future::time::sleep(PRESENCE_INTERVAL),
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
                    let event: ChatEvent = match event.try_into() {
                        Ok(event) => event,
                        Err(err) => {
                            warn!("received invalid message: {err}");
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
        secret_key: secret_key.clone(),
        nickname,
        sender,
        trigger_presence,
        _presence_task: Arc::new(presence_task),
    };
    Ok((sender, Box::pin(receiver)))
}

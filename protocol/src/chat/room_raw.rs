use std::sync::Arc;

use crate::chat::chat_controller::IChatRoomRaw;
use crate::chat::direct_message::{ChatDirectMessage, DirectMessageProtocol};
use crate::{
    chat::chat_const::CONNECT_TIMEOUT, chat::chat_ticket::ChatTicket,
    main_node::MainNode, user_identity::NodeIdentity,
};
use anyhow::{Context, Result};
use futures::{FutureExt, StreamExt};
use iroh::{NodeId, PublicKey};
use iroh_gossip::{
    net::{GossipEvent, GossipReceiver, GossipSender},
    proto::TopicId,
};
use n0_future::task::spawn;
use n0_future::task::AbortOnDropHandle;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct GossipChatRoom {
    own_node_id: NodeId,
    direct_message: DirectMessageProtocol<ChatDirectMessage>,
    topic_id: TopicId,
    gossip_send: Arc<RwLock<Option<GossipSender>>>,
    task: Arc<RwLock<Option<AbortOnDropHandle<()>>>>,
    msg_recv: Arc<RwLock<Option<tokio::sync::mpsc::Receiver<Arc<Vec<u8>>>>>>,
}

impl GossipChatRoom {
    pub async fn new(node: &MainNode, ticket: &ChatTicket) -> Result<Self> {
        // first, subscribe to direct messages
        let direct_message_recv = node.direct_message_recv.activate_cloned();

        // then, subscribe to gossip
        let mut bootstrap = ticket.bootstrap.clone();
        bootstrap.remove(&node.node_id());
        let bootstrap = bootstrap.into_iter().collect::<Vec<_>>();
        let have_bootstrap = bootstrap.len() > 0;
        let mut gossip_topic =
            node.gossip.subscribe(ticket.topic_id.clone(), bootstrap)?;
        if have_bootstrap {
            let _ = n0_future::time::timeout(
                CONNECT_TIMEOUT,
                gossip_topic.joined(),
            )
            .await;
        }
        let (gossip_send, gossip_recv) = gossip_topic.split();
        let gossip_send = Arc::new(RwLock::new(Some(gossip_send)));
        let (msg_send, msg_recv) =
            tokio::sync::mpsc::channel::<Arc<Vec<u8>>>(2048);
        let room = Self {
            own_node_id: node.node_id(),
            direct_message: node.chat_direct_message.clone(),
            topic_id: ticket.topic_id.clone(),
            gossip_send,
            task: Arc::new(RwLock::new(None)),
            msg_recv: Arc::new(RwLock::new(Some(msg_recv))),
        };
        {
            let task = Some(AbortOnDropHandle::new(spawn(async move {
                let _r = task_loop(
                    room.topic_id.clone(),
                    gossip_recv,
                    direct_message_recv,
                    msg_send,
                )
                .await;
                warn!("ZZZ: chat room task loop closed: {:?}", _r);
            })));
            *room.task.write().await = task;
        }
        Ok(room)
    }
}

async fn task_loop(
    topic_id: TopicId,
    mut gossip_recv: GossipReceiver,
    mut direct_message_recv: async_broadcast::Receiver<(
        PublicKey,
        ChatDirectMessage,
    )>,
    msg_send: tokio::sync::mpsc::Sender<Arc<Vec<u8>>>,
) -> Result<()> {
    loop {
        tokio::select! {
            msg = gossip_recv.next().fuse() => {
                let Some(msg) = msg else {
                    error!("gossip recv closed");
                    anyhow::bail!("gossip recv closed");
                };
                let Ok(msg) = msg else {
                    warn!("gossip recv error: {:?}", msg);
                    continue;
                };
                let msg = match msg {
                    iroh_gossip::net::Event::Gossip(
                        GossipEvent::Received(iroh_gossip::net::Message {
                            content, ..
                        })
                    )=> {
                        content
                    }
                    _ => {
                        continue;
                    }
                };
                msg_send.send(Arc::new(msg.to_vec())).await?;
            }
            msg = direct_message_recv.next().fuse() => {
                let Some((_from_pubkey, ChatDirectMessage(msg_topic_id, msg_data))) = msg else {
                    error!("direct message recv closed");
                    anyhow::bail!("direct message recv closed");
                };
                if msg_topic_id != topic_id {
                    continue;
                }
                msg_send.send(msg_data).await?;
            }
        }
    }
}

#[async_trait::async_trait]
impl IChatRoomRaw for GossipChatRoom {
    async fn shutdown(&self) -> anyhow::Result<()> {
        info!(
            "shutting down chat room, \n\t topic_id: {:?}",
            self.topic_id
        );
        {
            drop(self.task.write().await.take());
        }
        {
            self.gossip_send.write().await.take();
        }
        {
            self.msg_recv.write().await.take();
        }
        Ok(())
    }

    async fn broadcast_message(&self, message: Vec<u8>) -> anyhow::Result<()> {
        let mut gossip_send = self.gossip_send.write().await;
        let gossip_send = gossip_send.as_mut().context("room was shut down")?;
        gossip_send.broadcast(message.into()).await?;
        Ok(())
    }

    async fn direct_message(
        &self,
        to: NodeIdentity,
        message: Vec<u8>,
    ) -> anyhow::Result<()> {
        let message =
            ChatDirectMessage(self.topic_id.clone(), Arc::new(message));
        self.direct_message
            .send_direct_message(*to.node_id(), message)
            .await
    }

    async fn next_message(&self) -> anyhow::Result<Option<Arc<Vec<u8>>>> {
        let mut msg_recv = self.msg_recv.write().await;
        let msg_recv = msg_recv.as_mut().context("room was shut down")?;
        Ok(msg_recv.recv().await)
    }

    async fn join_peers(&self, peers: Vec<NodeId>) -> anyhow::Result<()> {
        let peers: Vec<PublicKey> = peers
            .into_iter()
            .filter(|p| *p != self.own_node_id)
            .collect();
        if peers.is_empty() {
            return Ok(());
        }
        let mut gossip_send = self.gossip_send.write().await;
        let gossip_send = gossip_send.as_mut().context("room was shut down")?;
        gossip_send.join_peers(peers).await?;
        Ok(())
    }
}

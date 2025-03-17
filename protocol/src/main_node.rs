use std::{sync::Arc, time::Duration};

use anyhow::Result;
use async_channel::Sender;
use iroh::{
    endpoint::{Connecting, Connection, RemoteInfo},
    protocol::{ProtocolHandler, Router},
    Endpoint, NodeId, SecretKey,
};
use iroh_gossip::{net::Gossip, ALPN as GOSSIP_ALPN};
use n0_future::{
    boxed::{BoxFuture, BoxStream},
    task::{self, AbortOnDropHandle},
    Stream, StreamExt,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tokio::sync::{broadcast, Notify};
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, info, warn};

use crate::{
    chat::{
        join_chat, ChatEvent, ChatEventStream, ChatSender, ChatTicket, Message, SignedMessage,
        PRESENCE_INTERVAL,
    },
    echo::Echo,
    global_matchmaker::GlobalMatchmaker,
};

#[derive(Debug, Clone)]
pub struct MainNode {
    secret_key: SecretKey,
    router: Router,
    gossip: Gossip,
    nickname: String,
}

impl MainNode {
    pub async fn spawn(
        nickname: String,
        secret_key: SecretKey,
        own_endpoint_node_id: Option<NodeId>,
    ) -> Result<Self> {
        let endpoint = Endpoint::builder()
            .secret_key(secret_key.clone())
            .discovery_n0()
            .alpns(vec![Echo::ALPN.to_vec(), GOSSIP_ALPN.to_vec()])
            .bind()
            .await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let echo = Echo::new(own_endpoint_node_id.unwrap_or(endpoint.node_id()));
        let router = Router::builder(endpoint)
            .accept(Echo::ALPN, echo)
            .accept(GOSSIP_ALPN, gossip.clone())
            .spawn()
            .await?;
        Ok(Self {
            router,
            secret_key,
            gossip,
            nickname,
        })
    }

    pub fn nickname(&self) -> &str {
        &self.nickname
    }

    pub fn endpoint(&self) -> &Endpoint {
        self.router.endpoint()
    }
    pub fn node_id(&self) -> NodeId {
        self.router.endpoint().node_id()
    }
    pub fn remote_info(&self) -> Vec<RemoteInfo> {
        self.router
            .endpoint()
            .remote_info_iter()
            .collect::<Vec<_>>()
    }

    pub async fn shutdown(&self) -> Result<()> {
        let _ = self.router.shutdown().await;
        self.gossip.shutdown().await;
        self.endpoint().close().await;
        Ok(())
    }

    /// Joins a chat channel from a ticket.
    ///
    /// Returns a [`ChatSender`] to send messages or change our nickname
    /// and a stream of [`Event`] items for incoming messages and other event.s
    pub fn join_chat(&self, ticket: &ChatTicket) -> Result<(ChatSender, ChatEventStream)> {
        join_chat(
            self.gossip.clone(),
            self.secret_key.clone(),
            self.nickname.clone(),
            ticket,
        )
    }
}

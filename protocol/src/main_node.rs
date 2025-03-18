use std::sync::Arc;

use anyhow::Result;
use iroh::{
    endpoint::RemoteInfo,
    protocol::{ProtocolHandler, Router},
    Endpoint, NodeId, SecretKey,
};
use iroh_gossip::{net::Gossip, ALPN as GOSSIP_ALPN};

use crate::{
    chat::{join_chat, ChatController, ChatTicket},
    echo::Echo,
    user_identity::{NodeIdentity, UserIdentity, UserIdentitySecrets},
};

#[derive(Clone)]
pub struct MainNode {
    node_secret_key: Arc<SecretKey>,
    router: Router,
    gossip: Gossip,
    node_identity: Arc<NodeIdentity>,
    user_secrets: Arc<UserIdentitySecrets>,
}

impl MainNode {
    pub async fn spawn(
        node_identity: Arc<NodeIdentity>,
        node_secret_key: Arc<SecretKey>,
        own_endpoint_node_id: Option<NodeId>,
        user_secrets: Arc<UserIdentitySecrets>,
    ) -> Result<Self> {
        assert!(node_secret_key.public() == *node_identity.node_id());
        let endpoint = Endpoint::builder()
            .secret_key(node_secret_key.as_ref().clone())
            .discovery_n0()
            .alpns(vec![Echo::ALPN.to_vec(), GOSSIP_ALPN.to_vec()])
            .bind()
            .await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let echo =
            Echo::new(own_endpoint_node_id.unwrap_or(endpoint.node_id()));
        let router = Router::builder(endpoint)
            .accept(Echo::ALPN, echo)
            .accept(GOSSIP_ALPN, gossip.clone())
            .spawn()
            .await?;
        Ok(Self {
            router,
            node_secret_key,
            gossip,
            node_identity,
            user_secrets,
        })
    }

    pub fn user(&self) -> &NodeIdentity {
        &self.node_identity
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
    pub fn node_identity(&self) -> &NodeIdentity {
        &self.node_identity
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
    pub fn join_chat(&self, ticket: &ChatTicket) -> Result<ChatController> {
        join_chat(
            self.gossip.clone(),
            self.node_secret_key.clone(),
            ticket,
            self.user_secrets.clone(),
            self.node_identity.clone(),
        )
    }
}

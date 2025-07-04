use std::sync::Arc;

use anyhow::Result;
use iroh::{
    discovery::pkarr::PkarrPublisher,
    endpoint::RemoteInfo,
    protocol::{ProtocolHandler, Router},
    Endpoint, NodeId, PublicKey, RelayMap, RelayNode, SecretKey,
};
use iroh_gossip::{net::Gossip, ALPN as GOSSIP_ALPN};
use tracing::{info, warn};

use crate::{
    _const::get_relay_domain,
    chat::{
        ChatController, ChatDirectMessage, DirectMessageProtocol,
        GossipChatRoom, CHAT_DIRECT_MESSAGE_ALPN,
    },
    chat_ticket::ChatTicket,
    echo::Echo,
    signed_message::{IChatRoomType, MessageSigner},
    sleep::SleepManager,
    user_identity::{NodeIdentity, UserIdentitySecrets},
};

#[derive(Debug, Clone)]
pub struct MainNode {
    pub(crate) router: Router,
    pub(crate) gossip: Gossip,
    pub(crate) node_identity: Arc<NodeIdentity>,
    pub(crate) sleep_manager: SleepManager,
    pub(crate) message_signer: MessageSigner,
    pub(crate) direct_message_recv:
        async_broadcast::InactiveReceiver<(PublicKey, ChatDirectMessage)>,
    pub(crate) chat_direct_message: DirectMessageProtocol<ChatDirectMessage>,
}

async fn create_endpoint(
    node_secret_key: Arc<SecretKey>,
) -> anyhow::Result<Endpoint> {
    let (relay_url, pkarr_url) = get_relay_domain();
    let relay_map = RelayMap::from_nodes([RelayNode {
        url: relay_url.parse().unwrap(),
        stun_only: false,
        stun_port: 31232,
        quic: None,
    }])
    .unwrap();
    let pkarr_publisher = PkarrPublisher::new(
        node_secret_key.as_ref().clone(),
        pkarr_url.parse().unwrap(),
    );

    // #[cfg(target_arch = "wasm32")]
    let discovery2 =
        iroh::discovery::pkarr::PkarrResolver::new(pkarr_url.parse().unwrap());
    // #[cfg(not(target_arch = "wasm32"))]
    // let discovery2 = iroh::discovery::dns::DnsDiscovery::new(
    //     "127.0.0.1".parse().unwrap()
    // );

    Endpoint::builder()
        .secret_key(node_secret_key.as_ref().clone())
        // .discovery_n0()
        .relay_mode(iroh::RelayMode::Custom(relay_map))
        .add_discovery(|_| Some(pkarr_publisher))
        .add_discovery(|_| Some(discovery2))
        .alpns(vec![
            Echo::ALPN.to_vec(),
            GOSSIP_ALPN.to_vec(),
            // DIRECT_MESSAGE_ALPN.to_vec(),
        ])
        .bind()
        .await
}

impl MainNode {
    pub async fn spawn(
        node_identity: Arc<NodeIdentity>,
        node_secret_key: Arc<SecretKey>,
        own_endpoint_node_id: Option<NodeId>,
        user_secrets: Arc<UserIdentitySecrets>,
        sleep_manager: SleepManager,
    ) -> Result<Self> {
        assert!(node_secret_key.public() == *node_identity.node_id());
        assert!(
            node_identity.user_id() == user_secrets.user_identity().user_id()
        );
        assert!(*node_identity.user_id() == user_secrets.secret_key().public());
        let message_signer = MessageSigner {
            node_secret_key: node_secret_key.clone(),
            user_secrets: user_secrets.clone(),
            node_identity: node_identity.clone(),
        };

        let endpoint = create_endpoint(node_secret_key.clone()).await?;
        let gossip = Gossip::builder().spawn(endpoint.clone()).await?;
        let echo = Echo::new(
            own_endpoint_node_id.unwrap_or(endpoint.node_id()),
            sleep_manager.clone(),
        );
        let (mut direct_message_send, mut direct_message_recv) =
            async_broadcast::broadcast(2048);
        direct_message_send.set_overflow(true);
        direct_message_recv.set_overflow(true);

        let chat_direct_message =
            DirectMessageProtocol::<ChatDirectMessage>::new(
                direct_message_send,
                sleep_manager.clone(),
                endpoint.clone(),
            );
        let router = Router::builder(endpoint.clone())
            .accept(Echo::ALPN, echo)
            .accept(GOSSIP_ALPN, gossip.clone())
            .accept(CHAT_DIRECT_MESSAGE_ALPN, chat_direct_message.clone())
            .spawn()
            .await?;

        Ok(Self {
            router,
            gossip,
            node_identity,
            sleep_manager,
            message_signer,
            direct_message_recv: direct_message_recv.deactivate(),
            chat_direct_message,
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
        info!("MainNode shutdown");
        let _ = self.router.shutdown().await;
        let _ = self.chat_direct_message.shutdown().await;
        self.gossip.shutdown().await;
        self.endpoint().close().await;
        Ok(())
    }
    /// Joins a chat channel from a ticket.
    ///
    /// Returns a [`ChatSender`] to send messages or change our nickname
    /// and a stream of [`Event`] items for incoming messages and other event.s
    pub async fn join_chat<T>(
        &self,
        ticket: &ChatTicket,
    ) -> Result<ChatController<T>>
    where
        T: IChatRoomType,
    {
        let mut ticket = ticket.clone();
        ticket.bootstrap.remove(&self.node_id());
        let room = match GossipChatRoom::new(self, &ticket).await {
            Ok(room) => room,
            Err(e) => {
                warn!("Failed to join GossipChatRoom: {e}");
                return Err(e);
            }
        };
        let cc = ChatController::<T>::new(
            ticket,
            Arc::new(room),
            self.message_signer.clone(),
            self.sleep_manager.clone(),
            self.node_identity().clone(),
        );
        Ok(cc)
    }
}

use std::sync::Arc;

use anyhow::Result;
use iroh::{
    discovery::pkarr::PkarrPublisher, endpoint::RemoteInfo, protocol::{ProtocolHandler, Router}, Endpoint, NodeId, RelayMap, RelayNode, SecretKey
};
use iroh_gossip::{net::Gossip, ALPN as GOSSIP_ALPN};
use matchbox_socket::{PeerEvent, PeerId};
use tracing::info;

use crate::{
    _matchbox_signal::{
        DirectMessageProtocol, MatchboxSignallerHolder, PeerTracker,
        DIRECT_MESSAGE_ALPN,
    },
    _const::IROH_RELAY_DOMAIN,
    chat::ChatController,
    chat_ticket::ChatTicket,
    echo::Echo,
    signed_message::{IChatRoomType, MessageSigner},
    sleep::SleepManager,
    user_identity::{NodeIdentity, UserIdentitySecrets},
};

#[derive(Clone)]
pub struct MainNode {
    router: Router,
    gossip: Gossip,
    node_identity: Arc<NodeIdentity>,
    sleep_manager: SleepManager,
    matchbox_signal_builder: MatchboxSignallerHolder,
    message_signer: MessageSigner,
}

async fn create_endpoint(
    node_secret_key: Arc<SecretKey>,
) -> anyhow::Result<Endpoint> {
    let relay_url = format!("http://{}:8084", IROH_RELAY_DOMAIN);
    let pkarr_url = format!("http://{}:18080/pkarr", IROH_RELAY_DOMAIN);
    let relay_map = RelayMap::from_nodes([
        RelayNode {
            url: relay_url.parse().unwrap(),
            stun_only: false,
            stun_port: 31232,
            quic: None,
        },
    ]).unwrap();
    let pkarr_publisher = PkarrPublisher::new(
        node_secret_key.as_ref().clone(), 
        pkarr_url.parse().unwrap());

    // #[cfg(target_arch = "wasm32")]
    let discovery2 =  iroh::discovery::pkarr::PkarrResolver::new(
        pkarr_url.parse().unwrap()
    );
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
            DIRECT_MESSAGE_ALPN.to_vec(),
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
        matchbox_id: PeerId,
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
        let (direct_message_send, direct_message_recv) =
            async_broadcast::broadcast(2048);
        let direct_message = DirectMessageProtocol::<PeerEvent> {
            sender: direct_message_send,
            sleep_manager: sleep_manager.clone(),
        };
        let router = Router::builder(endpoint.clone())
            .accept(Echo::ALPN, echo)
            .accept(GOSSIP_ALPN, gossip.clone())
            .accept(DIRECT_MESSAGE_ALPN, direct_message)
            .spawn()
            .await?;
        let matchbox_signal_builder = MatchboxSignallerHolder {
            matchbox_id,
            iroh_id: endpoint.node_id().clone(),
            endpoint: endpoint.clone(),
            gossip: gossip.clone(),
            direct_message_recv: direct_message_recv.deactivate(),
            message_signer: message_signer.clone(),
            peer_tracker: PeerTracker::new(),
            sleep_manager: sleep_manager.clone(),
        };
        Ok(Self {
            router,
            gossip,
            node_identity,
            sleep_manager,
            matchbox_signal_builder,
            message_signer,
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
        let room = self
            .matchbox_signal_builder
            .open_socket(ticket.clone())
            .await?;

        Ok(ChatController::<T>::new(
            Arc::new(room),
            self.message_signer.clone(),
            self.sleep_manager.clone(),
            self.matchbox_signal_builder.peer_tracker.clone(),
        ))
    }
}

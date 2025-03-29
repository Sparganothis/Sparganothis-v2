use n0_future::time::Instant;
use std::{collections::BTreeMap, sync::Arc};

use iroh::PublicKey;
use matchbox_socket::PeerId;
use tokio::sync::RwLock;

use crate::{_const::PRESENCE_EXPIRATION, user_identity::NodeIdentity};

#[derive(Debug, Clone)]
pub struct PeerTracker {
    inner: Arc<RwLock<PeerTrackerInner>>,
}

impl PeerTracker {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(PeerTrackerInner::new())),
        }
    }
    pub async fn add_peer(&self, node_identity: NodeIdentity) {
        self.inner.write().await.add_peer(node_identity);
    }
    pub async fn confirm_peer(&self, node_identity: NodeIdentity) {
        self.inner.write().await.confirm_peer(node_identity);
    }
    pub async fn is_confirmed(&self, node_identity: NodeIdentity) -> bool {
        self.inner.read().await.is_confirmed(node_identity)
    }
    pub async fn get_peer_by_matchbox_id(
        &self,
        peer_id: PeerId,
    ) -> Option<NodeIdentity> {
        self.inner
            .read()
            .await
            .matchbox_to_iroh
            .get(&peer_id)
            .map(|(node_identity, _)| node_identity.clone())
    }
    pub async fn _get_peer_by_iroh_id(
        &self,
        iroh_id: PublicKey,
    ) -> Option<NodeIdentity> {
        self.inner
            .read()
            .await
            .iroh_to_matchbox
            .get(&iroh_id)
            .map(|(node_identity, _)| node_identity.clone())
    }
    pub async fn expired_peers(&self) -> Vec<NodeIdentity> {
        self.inner.write().await.expired_peers()
    }
    pub async fn drop_peers(&self, dead_peers: Vec<NodeIdentity>) {
        self.inner.write().await.drop_peers(dead_peers);
    }
    pub async fn peers(&self) -> Vec<NodeIdentity> {
        self.inner
            .read()
            .await
            .matchbox_to_iroh
            .values()
            .map(|(node_identity, _)| node_identity.clone())
            .collect()
    }
}

#[derive(Debug)]
struct PeerTrackerInner {
    matchbox_to_iroh: BTreeMap<PeerId, (NodeIdentity, Instant)>,
    confirmed: BTreeMap<PeerId, bool>,
    iroh_to_matchbox: BTreeMap<PublicKey, (NodeIdentity, Instant)>,
}

impl PeerTrackerInner {
    fn new() -> Self {
        Self {
            matchbox_to_iroh: BTreeMap::new(),
            confirmed: BTreeMap::new(),
            iroh_to_matchbox: BTreeMap::new(),
        }
    }
    fn add_peer(&mut self, node_identity: NodeIdentity) {
        self.matchbox_to_iroh.insert(
            node_identity.matchbox_id().clone(),
            (node_identity.clone(), Instant::now()),
        );
        self.iroh_to_matchbox.insert(
            node_identity.user_id().clone(),
            (node_identity, Instant::now()),
        );
    }
    fn confirm_peer(&mut self, node_identity: NodeIdentity) {
        self.add_peer(node_identity.clone());
        self.confirmed
            .insert(node_identity.matchbox_id().clone(), true);
    }
    fn is_confirmed(&self, node_identity: NodeIdentity) -> bool {
        *self
            .confirmed
            .get(&node_identity.matchbox_id().clone())
            .unwrap_or(&false)
    }
    /// Returns the list of dead peers
    fn expired_peers(&mut self) -> Vec<NodeIdentity> {
        let now = Instant::now();
        let dead_peers = self
            .matchbox_to_iroh
            .iter()
            .filter(|(_, (_, timestamp))| {
                now.duration_since(*timestamp) >= PRESENCE_EXPIRATION
            })
            .map(|(_, (node_id, _))| *node_id)
            .collect::<Vec<_>>();
        dead_peers
    }

    fn drop_peers(&mut self, dead_peers: Vec<NodeIdentity>) {
        for node in dead_peers.iter() {
            let peer_id = node.matchbox_id().clone();
            let node_id = node.user_id().clone();
            tracing::info!(
                "Removing dead peer connection: {peer_id} -> {node_id}"
            );
            self.matchbox_to_iroh.remove(&peer_id);
            self.iroh_to_matchbox.remove(&node_id);
            self.confirmed.remove(&peer_id);
        }
    }
}

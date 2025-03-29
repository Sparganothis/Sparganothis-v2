use crate::{
    chat::IChatRoomRaw, sleep::SleepManager, user_identity::NodeIdentity,
};
use async_broadcast::Sender;
use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    FutureExt, SinkExt, StreamExt,
};
use iroh::PublicKey;
use matchbox_socket::async_trait::async_trait;
use matchbox_socket::{Packet, PeerId, PeerState};
use n0_future::task::AbortOnDropHandle;
use tokio::sync::Mutex;

use super::PeerTracker;

#[derive(Debug)]
pub struct MatchboxRoom {
    pub(crate) sender: Mutex<UnboundedSender<(PeerId, Packet)>>,
    pub(crate) recv: Mutex<UnboundedReceiver<(PeerId, Packet)>>,
    pub(crate) events: Mutex<UnboundedReceiver<(PeerId, PeerState)>>,
    pub(crate) _task_events: AbortOnDropHandle<()>,
    pub(crate) _task_loop: AbortOnDropHandle<()>,
    pub(crate) join_commands: Mutex<Sender<Vec<PublicKey>>>,
    pub(crate) peer_tracker: PeerTracker,
    pub(crate) sleep_manager: SleepManager,
}

#[async_trait]
impl IChatRoomRaw for MatchboxRoom {
    async fn broadcast_message(&self, message: Vec<u8>) -> anyhow::Result<()> {
        let mut sender = self.sender.lock().await;
        for peer_id in self.peer_tracker.peers().await {
            if !self.peer_tracker.is_confirmed(peer_id).await {
                continue;
            };
            sender
                .send((*peer_id.matchbox_id(), Packet::from(message.clone())))
                .await?;
        }
        Ok(())
    }
    async fn direct_message(
        &self,
        to: NodeIdentity,
        message: Vec<u8>,
    ) -> anyhow::Result<()> {
        let Some(to2) = self
            .peer_tracker
            .get_peer_by_matchbox_id(*to.matchbox_id())
            .await
        else {
            return Err(anyhow::anyhow!("Peer not found"));
        };
        let mut sender = self.sender.lock().await;
        sender
            .send((*to2.matchbox_id(), Packet::from(message.clone())))
            .await?;
        Ok(())
    }
    async fn next_message(&self) -> Option<(NodeIdentity, Vec<u8>)> {
        let recv = { self.recv.lock().await.next().fuse().await };
        self.sleep_manager.wake_up();
        let (peer_id, packet) = recv?;
        let peer = self.peer_tracker.get_peer_by_matchbox_id(peer_id).await?;
        Some((peer, packet.to_vec()))
    }
    async fn next_peer_event(&self) -> Option<(NodeIdentity, PeerState)> {
        let recv = { self.events.lock().await.next().fuse().await };
        self.sleep_manager.wake_up();
        let (peer_id, peer_state) = recv?;
        let peer = self.peer_tracker.get_peer_by_matchbox_id(peer_id).await?;
        Some((peer, peer_state))
    }
    async fn join_peers(&self, peers: Vec<PublicKey>) -> anyhow::Result<()> {
        let join_commands = self.join_commands.lock().await;
        join_commands.broadcast(peers).await?;
        Ok(())
    }
    async fn shutdown(&self) -> anyhow::Result<()> {
        {
            let sender = self.sender.lock().await;
            sender.close_channel();
        }
        {
            let join_commands = self.join_commands.lock().await;
            join_commands.close();
        }
        Ok(())
    }
    async fn peer_tracker(&self) -> PeerTracker {
        self.peer_tracker.clone()
    }
}

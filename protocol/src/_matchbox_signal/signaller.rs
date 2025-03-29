use std::sync::Arc;

use anyhow::Context;
use async_broadcast::InactiveReceiver;
use futures::{channel::mpsc::unbounded, FutureExt, SinkExt};
use iroh::{Endpoint, PublicKey};
use iroh_gossip::net::{
    Event, Gossip, GossipEvent, GossipReceiver, GossipSender, Message,
};
use matchbox_socket::{
    async_trait::async_trait, error::SignalingError, PeerEvent, PeerId,
    PeerRequest, PeerSignal, Signaller, SignallerBuilder, WebRtcSocket,
};
use n0_future::{task::spawn, task::AbortOnDropHandle, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::{
    _const::PRESENCE_INTERVAL,
    _matchbox_signal::{
        direct_message::send_direct_message, peer_tracker::PeerTracker,
    },
    chat_ticket::ChatTicket,
    signed_message::{MessageSigner, SignedMessage, WireMessage},
    sleep::SleepManager,
};

use super::{ice_servers, MatchboxRoom};

#[derive(Debug, Clone)]
pub struct MatchboxSignallerHolder {
    pub(crate) gossip: Gossip,
    pub(crate) endpoint: Endpoint,
    pub(crate) matchbox_id: PeerId,
    pub(crate) iroh_id: PublicKey,
    pub(crate) direct_message_recv:
        async_broadcast::InactiveReceiver<(PublicKey, WireMessage<PeerEvent>)>,
    pub(crate) message_signer: MessageSigner,
    pub(crate) peer_tracker: PeerTracker,
    pub(crate) sleep_manager: SleepManager,
}

#[derive(Debug)]
struct MatchboxSignallerBuilder {
    holder: MatchboxSignallerHolder,
    join_commands: InactiveReceiver<Vec<PublicKey>>,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl SignallerBuilder for MatchboxSignallerBuilder {
    async fn new_signaller(
        &self,
        _attempts: Option<u16>,
        room_url: String,
    ) -> Result<Box<dyn Signaller>, SignalingError> {
        let room_url = serde_json::from_str::<ChatTicket>(&room_url)
            .map_err(to_user_error)?;
        for i in 0.._attempts.unwrap_or(3) {
            match self
                .holder
                .try_new_signaller(
                    room_url.clone(),
                    self.join_commands.activate_cloned(),
                )
                .await
            {
                Ok(signaller) => {
                    return Ok(Box::new(signaller));
                }
                Err(e) => {
                    warn!("Failed to connect to gossip: {e:#?}");
                    if i == _attempts.unwrap_or(1) - 1 {
                        return Err(to_user_error(e));
                    }
                }
            }
        }
        unreachable!()
    }
}

fn to_user_error<E: std::fmt::Debug>(e: E) -> SignalingError {
    SignalingError::UserImplementationError(format!("{e:#?}"))
}

impl MatchboxSignallerHolder {
    pub async fn open_socket(
        &self,
        chat_ticket: ChatTicket,
    ) -> anyhow::Result<MatchboxRoom> {
        let (mut join_commands, mut join_commands_receiver) =
            async_broadcast::broadcast(1024);
        join_commands.set_overflow(true);
        join_commands_receiver.set_overflow(true);

        let s = Arc::new(MatchboxSignallerBuilder {
            holder: self.clone(),
            join_commands: join_commands_receiver.deactivate(),
        });
        let room_url = serde_json::to_string(&chat_ticket)?;
        let (mut socket, loop_fut) = WebRtcSocket::builder(room_url)
            .signaller_builder(s)
            .add_reliable_channel()
            .ice_server(ice_servers())
            .build();

        let chan0 = socket.take_channel(0)?;
        let (sender, recv) = chan0.split();
        let (events_send, events_recv) = unbounded();
        let sleep_manager = self.sleep_manager.clone();
        let _task_events = AbortOnDropHandle::new(spawn(async move {
            let mut events_send = events_send;
            while let Some(event) = socket.next().await {
                if let Err(e) = events_send.send(event).await {
                    error!("Error sending event: {e:#?}");
                    break;
                }
                sleep_manager.wake_up();
            }
            warn!("Events task exited");
        }));
        let _task_loop = AbortOnDropHandle::new(spawn(async move {
            let r = loop_fut.await;
            if let Err(e) = r {
                error!("Error in loop task: {e:#?}");
            }
        }));
        Ok(MatchboxRoom {
            sender: Mutex::new(sender),
            recv: Mutex::new(recv),
            events: Mutex::new(events_recv),
            _task_events,
            _task_loop,
            join_commands: Mutex::new(join_commands),
            peer_tracker: self.peer_tracker.clone(),
            sleep_manager: self.sleep_manager.clone(),
        })
    }

    async fn try_new_signaller(
        &self,
        room_url: ChatTicket,
        join_commands: async_broadcast::Receiver<Vec<PublicKey>>,
    ) -> anyhow::Result<IrohGossipSignaller> {
        info!("Creating new signaller");
        let mut bootstrap = room_url.bootstrap.clone();
        bootstrap.remove(&self.iroh_id);
        let mut gossip_topic = self
            .gossip
            .subscribe(room_url.topic_id, bootstrap.into_iter().collect())?;
        info!("Joining gossip topic...");
        gossip_topic.joined().await?;
        info!("Connected to gossip topic.");
        let (gossip_send, gossip_recv) = gossip_topic.split();

        let (req_send, req_recv) = tokio::sync::mpsc::channel(2048);
        let (event_send, event_recv) = tokio::sync::mpsc::channel(2048);

        let _task = self
            .clone()
            .spawn_task(
                gossip_recv,
                gossip_send,
                req_recv,
                event_send,
                self.direct_message_recv.activate_cloned(),
                join_commands,
                self.peer_tracker.clone(),
            )
            .then(|r| async move {
                match r {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Error in signaller task: \n {e:#?}. \n Signaller task exit.");
                        Err(e)
                    }
                }
            });
        let _task = AbortOnDropHandle::new(spawn(_task));
        Ok(IrohGossipSignaller {
            recv: event_recv,
            send: req_send,
            _task,
        })
    }

    async fn spawn_task(
        self,
        // gossip msg from other nodes
        mut gossip_recv: GossipReceiver,
        // gossip msg into other nodes
        gossip_send: GossipSender,
        // sent requests from client
        mut req_recv: tokio::sync::mpsc::Receiver<PeerRequest>,
        // send events to client
        event_send: tokio::sync::mpsc::Sender<PeerEvent>,
        // get direct messages from other clients
        mut direct_message_recv: async_broadcast::Receiver<(
            PublicKey,
            WireMessage<PeerEvent>,
        )>,
        // join more peers in gossip
        mut join_commands: async_broadcast::Receiver<Vec<PublicKey>>,
        // hold all the peers we know about
        peer_tracker: PeerTracker,
    ) -> anyhow::Result<()> {
        info!("Spawning signaller task");

        // send AssignedId into client
        event_send
            .send(PeerEvent::IdAssigned(self.matchbox_id))
            .await?;

        // send gossip message to ensure other nodes have our iroh id
        self.send_gossip_message(&gossip_send).await?;

        let mut refresh_interval = n0_future::time::interval(PRESENCE_INTERVAL);

        loop {
            tokio::select! {
                join_command = join_commands.next().fuse() => {
                    if let Some(join_command) = join_command {
                        if join_command.is_empty() {
                            continue;
                        }
                        info!("Joining peers: {join_command:#?}");
                        if let Err(e) = gossip_send.join_peers(join_command).await {
                            error!("Error joining peers: {e:#?}");
                        }
                    }
                }
                gossip_msg = gossip_recv.next().fuse() => {
                    self.sleep_manager.wake_up();
                    debug!("Received gossip message.");
                    // receive gossip messages and send NewPeer events
                    let Some(Ok(gossip_msg)) = gossip_msg else {
                        anyhow::bail!("Gossip receiver stream problem: {:#?}", gossip_msg);
                    };
                    match gossip_msg {
                        Event::Lagged => {
                            // Iroh will close the receiver after this event, so we can exit here.
                            anyhow::bail!("Gossip receiver lagged");
                        }
                        Event::Gossip(GossipEvent::Received(Message { content: gossip_msg, ..})) => {
                            let WireMessage {
                                from,
                                ..
                            } = SignedMessage::verify_and_decode::<()>(&gossip_msg)?;
                            let matchbox_id = from.matchbox_id().clone();
                            let iroh_id = from.user_id().clone();
                            let is_new =  peer_tracker.get_peer_by_matchbox_id(matchbox_id).await.is_none();
                            peer_tracker.add_peer(from).await;
                            if is_new {
                                info!("New peer connection:
                                - new peer matchbox ID {matchbox_id}
                                - new peer iroh ID {iroh_id}");

                                if matchbox_id < self.matchbox_id {
                                    info!("Sending NewPeer event for smaller id");
                                    event_send.send(PeerEvent::NewPeer(matchbox_id)).await?;
                                } else {
                                    info!("Not sending NewPeer event for larger id");
                                }
                                // on new peer, send gossip message to ensure new peer has our iroh id
                                self.send_gossip_message(&gossip_send).await?;
                            }
                        }
                        _ => {
                            // ignore other gossip events, they're not relevant here
                        }
                    }
                },
                req_msg = req_recv.recv().fuse() => {
                    debug!("Client message request.");
                    // on client request, either send keep alive to gossip, or send direct message with the signal to peer
                    let Some(req_msg) = req_msg else {
                        anyhow::bail!("Request receiver stream problem: {:#?}", req_msg);
                    };
                    match req_msg {
                        PeerRequest::KeepAlive => {
                            // send keep alive to gossip, containing all of our IDs
                            self.send_gossip_message(&gossip_send).await?;
                        }
                        PeerRequest::Signal { receiver, data } => {
                            // send direct message to MatchboxSignalProtocol
                            match self.send_direct_message(receiver, data).await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Error sending direct message: {e:#?}");
                                }
                            }
                        }
                    }
                },
                direct_msg = direct_message_recv.next().fuse() => {
                    debug!("Received direct message");
                    self.sleep_manager.wake_up();
                    let Some((from_iroh_id, event)) = direct_msg else {
                        anyhow::bail!("Direct message receiver stream problem: {:#?}", direct_msg);
                    };
                    // check message is from who it said it is
                    if let PeerEvent::Signal {sender, ..} = &event.message {
                        if peer_tracker.get_peer_by_matchbox_id(*sender).await.map(|peer| *peer.node_id()) == Some(from_iroh_id) {
                            debug!("
                            Received direct message:
                                From Matchbox ID: {sender}
                                From Iroh ID: {from_iroh_id}
                                Event: {event:#?}
                            ");
                            event_send.send(event.message.clone()).await?;
                        } else {
                            warn!("Received message from {from_iroh_id} with wrong sender: {sender}");
                        }
                    } else {
                        warn!("Received message from {from_iroh_id} with wrong event type: {event:#?}");
                    }
                }
                _ = refresh_interval.tick().fuse() => {
                    debug!("Sending gossip message");
                    self.send_gossip_message(&gossip_send).await?;
                    // check for stale connections and send PeerLeft events
                    let dead_peers = peer_tracker.expired_peers().await;
                    peer_tracker.drop_peers(dead_peers.clone()).await;
                    for peer in dead_peers.iter() {
                        info!("Removing dead peer connection: {peer:?}");
                        event_send.send(PeerEvent::PeerLeft(*peer.matchbox_id())).await?;
                    }
                }
            }
        }
    }

    async fn send_gossip_message(
        &self,
        gossip_send: &GossipSender,
    ) -> anyhow::Result<()> {
        debug!("Sending gossip message");
        let message = ();
        let message = self.message_signer.sign_and_encode(message)?;
        gossip_send.broadcast(message.into()).await?;
        Ok(())
    }

    async fn send_direct_message(
        &self,
        receiver: PeerId,
        data: PeerSignal,
    ) -> anyhow::Result<()> {
        let event = PeerEvent::Signal {
            sender: self.matchbox_id,
            data,
        };
        let target_node_id = *self
            .peer_tracker
            .get_peer_by_matchbox_id(receiver)
            .await
            .with_context(|| format!("No known connection to peer {receiver}"))?
            .node_id();
        debug!(
            "
        Sending direct message to:
            Matchbox ID: {receiver} 
            Iroh     ID: {target_node_id}
            Event: {event:#?}
        "
        );

        send_direct_message(
            &self.endpoint,
            target_node_id,
            event,
            &self.message_signer,
        )
        .await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct GossipMessage {
    matchbox_id: PeerId,
    iroh_id: PublicKey,
    timestamp: u128,
}

struct IrohGossipSignaller {
    recv: tokio::sync::mpsc::Receiver<PeerEvent>,
    send: tokio::sync::mpsc::Sender<PeerRequest>,
    _task: AbortOnDropHandle<Result<(), anyhow::Error>>,
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Signaller for IrohGossipSignaller {
    async fn send(
        &mut self,
        request: PeerRequest,
    ) -> Result<(), SignalingError> {
        debug!("\n\nSignaller: Sending request: {request:#?}\n\n");
        self.send.send(request).await.map_err(to_user_error)?;
        Ok(())
    }
    async fn next_message(&mut self) -> Result<PeerEvent, SignalingError> {
        let Some(message) = self.recv.recv().await else {
            info!("\n\nSignaller: Stream exhausted\n\n");
            return Err(SignalingError::StreamExhausted);
        };
        debug!("\n\n Signaller: Received message: {message:#?}\n\n");
        Ok(message)
    }
}

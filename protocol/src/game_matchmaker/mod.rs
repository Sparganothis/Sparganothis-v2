use std::{collections::HashSet, sync::Arc};

use anyhow::Context;
use game::api::game_match::{GameMatch, GameMatchType};
use n0_future::task::AbortOnDropHandle;
use rand::Rng;
use tokio::sync::{mpsc::channel, Mutex};
use tracing::{info, warn};

use crate::{chat::{ChatController, IChatController, IChatReceiver, IChatSender}, global_chat::{GlobalChatMessageContent, GlobalChatMessageType, MatchHandshakeType, MatchmakingMessage}, global_matchmaker::GlobalMatchmaker, user_identity::NodeIdentity};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct MatchmakeRandomId ( u16);

impl MatchmakeRandomId {
    pub fn random() -> Self {
        MatchmakeRandomId ( rand::thread_rng().gen())
    }
}


pub async fn find_game(
    match_type: GameMatchType,
    global_chat: ChatController<GlobalChatMessageType>,
    timeout: std::time::Duration,
) -> anyhow::Result<NodeIdentity> {
    let own_node_id = global_chat.node_identity();
    warn!(">>> find game: {:?}", &match_type);
    let (tx, mut rx) = channel(1);
    let m = GameMatchmaker::new(match_type.clone(), global_chat, tx, own_node_id);
    let mut m2 = m.clone();
    let _task = n0_future::task::spawn(async move {
        warn!(">>> start loop: {:?}",& match_type);
        let _r = m2.run_loop().await;
        warn!(">>> stop loop: {:?}", &match_type);
    });
    let _task = AbortOnDropHandle::new(_task);
    
    warn!(">>> broadcast lfg lfg");
    m.broadcast_lfg().await?;
    let m = n0_future::time::timeout(timeout, rx.recv()).await?.context("matchmaker tx dropped")?;
    // _task.abort();
    
    warn!(">>> RESULT RECV!!!!!");
    Ok(m)
}


#[derive(Clone, Debug)]
struct GameMatchmaker {
    own_node_id: NodeIdentity,
    match_type: GameMatchType,
    global_chat: ChatController<GlobalChatMessageType>,
    sender: tokio::sync::mpsc::Sender<NodeIdentity>,
    state: GameMatchmakerState,
    rando: MatchmakeRandomId,
}

impl GameMatchmaker {

    async fn broadcast_lfg(&self) -> anyhow::Result<()> {
        let m = GlobalChatMessageContent::MatchmakingMessage {
            msg: MatchmakingMessage::LFG {
                 match_type: self.match_type.clone() ,
                 rando: self.rando
                }
        };
        let _ = self.global_chat.sender().broadcast_message(m).await?;
        Ok(())
    }

    fn new(match_type: GameMatchType,
         global_chat: ChatController<GlobalChatMessageType>,
         sender: tokio::sync::mpsc::Sender<NodeIdentity>,
         node_id:NodeIdentity    
        )
    -> Self {
        let state = GameMatchmakerState {
            result: None,
            blacklist: HashSet::new(),
        };
        let rando = MatchmakeRandomId::random();
        Self {
            match_type,
            global_chat,
            sender,
            state,
            own_node_id: node_id,
            rando
        }
    }

    async fn run_loop(&mut self) -> anyhow::Result<()> {
        let recv = self.global_chat.receiver().await;

        while let Some(received_message) = recv.next_message().await {
            let content = received_message.message;
            match content {
                GlobalChatMessageContent::MatchmakingMessage {
                    ref msg,
                } => {
                    match &msg {
                        MatchmakingMessage::LFG { match_type, rando: lfg_rando } => {
                            self.on_lfg_message(
                                match_type.clone(),
                                received_message.from,
                                *lfg_rando
                            ).await;
                        }
                        MatchmakingMessage::Handshake {
                            match_type,
                            handshake_type,
                            rando
                        } => {
                            self.on_handshake_message(
                                match_type.clone(),
                                received_message.from,
                                handshake_type.clone(),
                                *rando
                            ).await;
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn on_lfg_message(
        &mut self,
        lfg_match_type: GameMatchType, 
        message_from: NodeIdentity, 
        lfg_rando: MatchmakeRandomId,
    ) {
        if self.own_node_id == message_from {
            return;
        }
        info!("recv LFG!");
        if lfg_match_type != self.match_type {
            return;
        }
        if self.state.blacklist.contains(&message_from) {
            return;
        }
        if self.state.result.is_some() {
            return;
        }
        let reply =
            GlobalChatMessageContent::handshake_request(self.match_type.clone(), lfg_rando);
        self.send_direct_message(message_from, reply).await;
        info!("send request");
    }
    async fn on_handshake_message(
        &mut self,
        match_type: GameMatchType, 
        from: NodeIdentity, 
        hand_type: MatchHandshakeType,
        hs_rando: MatchmakeRandomId,
    ) {
        if self.own_node_id == from {
            return;
        }
        if match_type != self.match_type {
            return;
        }
        if self.state.blacklist.contains(&from) {
            return;
        }
        match hand_type {
            MatchHandshakeType::HandshakeRequest => {
                if self.rando != hs_rando {
                    warn!("WRONG RANDO FROM MatchHandshakeType::HandshakeRequest ==> IGNORE");
                    self.state.blacklist.insert(from);
                    return;
                }
                info!("get request");
                if self.state.result.is_some() {
                    // answer NO
                    let no = GlobalChatMessageContent::handshake_no(match_type, hs_rando);
                    self.send_direct_message(from, no).await;
                    info!("send no");
                    return;
                }
                let yes = GlobalChatMessageContent::handshake_yes(match_type, hs_rando);
                self.send_direct_message(from, yes).await;
                info!("send yes");
                self.on_result(from).await;
            }
            MatchHandshakeType::AnswerYes => {
                info!("get yes");
                self.on_result(from).await;
            }
            MatchHandshakeType::AnswerNo => {
                info!("get no");
                self.state.blacklist.insert(from);
            }
        }
    }

    async fn send_direct_message(&self, to: NodeIdentity, x: GlobalChatMessageContent) {
        let _ = self.global_chat.sender().direct_message(to, x).await;
    }

    async fn on_result(&mut self, result: NodeIdentity) {
        self.state.result = Some(result);
        let _ = self.sender.send(result).await;
    }
}

#[derive(Debug, Clone)]
struct GameMatchmakerState  {
    result: Option<NodeIdentity>,
    blacklist: HashSet<NodeIdentity>,
}
use std::{collections::HashSet, time::Duration};

use anyhow::Context;
use game::{api::game_match::{GameMatch, GameMatchType}, tet::get_random_seed, timestamp::get_timestamp_now_ms};
use n0_future::task::AbortOnDropHandle;
use rand::Rng;
use tokio::sync::mpsc::channel;
use tracing::{info, warn};

use crate::{
    chat::{ChatController, IChatController, IChatReceiver, IChatSender},
    global_chat::{
        GlobalChatMessageContent, GlobalChatMessageType, MatchHandshakeType,
        MatchmakingMessage,
    },
    user_identity::NodeIdentity,
};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct MatchmakeRandomId(u16);

impl MatchmakeRandomId {
    pub fn random() -> Self {
        MatchmakeRandomId(rand::thread_rng().gen())
    }
}

pub async fn find_game(
    match_type: GameMatchType,
    global_chat: ChatController<GlobalChatMessageType>,
    timeout: std::time::Duration,
    attempts: u8,
) -> anyhow::Result<GameMatch<NodeIdentity>> {
    for _i in 0..attempts-1 {
        if let Ok(game) = find_game_one_attempt(match_type.clone(), global_chat.clone(), timeout).await {
            return Ok(game);
        }
        n0_future::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    find_game_one_attempt(match_type, global_chat, timeout).await
}


 async fn find_game_one_attempt(
    match_type: GameMatchType,
    global_chat: ChatController<GlobalChatMessageType>,
    timeout: std::time::Duration,
) -> anyhow::Result<GameMatch<NodeIdentity>>
{
    let own_node_id = global_chat.node_identity();
    warn!(">>> find game: {:?}", &match_type);
    let (tx, mut rx) = channel(1);
    let m =
        GameMatchmaker::new(match_type.clone(), global_chat, tx, own_node_id);
    let mut m2 = m.clone();
    let _task = n0_future::task::spawn(async move {
        warn!(">>> start task loop: {:?}", &match_type);
        let _r = m2.run_loop().await;
        warn!(">>> stop task loop: {:?}", &match_type);
    });
    let _task_main = AbortOnDropHandle::new(_task);

    let _task2 = n0_future::task::spawn(async move {
        info!(">>> start task broadcast lfg");
        for i in 0..5 {
            warn!(">>> broadcast lfg lfg {i}");
            if let Err(e) = m.broadcast_lfg().await {
                warn!("broadcast lfg {i} failed: {e}");
            }
            n0_future::time::sleep(timeout / 5).await;
        }
        info!(">>> stop task broadcast lfg");
    });
    let _task2 = AbortOnDropHandle::new(_task2);

    let m = n0_future::time::timeout(timeout, rx.recv())
        .await?
        .context("matchmaker tx dropped")?;
    // ensure we send the final ping pongs
    n0_future::time::sleep(Duration::from_millis(150)).await;
    drop(_task_main);
    drop(_task2);

    warn!(">>> RESULT RECV!!!!!");
    Ok(m)
}

#[derive(Clone, Debug)]
struct GameMatchmaker {
    own_node_id: NodeIdentity,
    match_type: GameMatchType,
    global_chat: ChatController<GlobalChatMessageType>,
    sender: tokio::sync::mpsc::Sender<GameMatch<NodeIdentity>>,
    state: GameMatchmakerState,
    rando: MatchmakeRandomId,
}

impl GameMatchmaker {
    async fn broadcast_lfg(&self) -> anyhow::Result<()> {
        let m = GlobalChatMessageContent::MatchmakingMessage {
            msg: MatchmakingMessage::LFG {
                match_type: self.match_type.clone(),
                rando: self.rando,
            },
        };
        if let Err(e) = self.global_chat.sender().broadcast_message(m).await {
            warn!("broadcast_lfg failed: {e}");
        }
        Ok(())
    }

    fn new(
        match_type: GameMatchType,
        global_chat: ChatController<GlobalChatMessageType>,
        sender: tokio::sync::mpsc::Sender<GameMatch<NodeIdentity>>,
        node_id: NodeIdentity,
    ) -> Self {
        let state = GameMatchmakerState {
            result_opponent: None,
            game_match: None,
            blacklist: HashSet::new(),
        };
        let rando = MatchmakeRandomId::random();
        warn!("GameMatchmaker::new(rando = {:?})", rando);
        Self {
            match_type,
            global_chat,
            sender,
            state,
            own_node_id: node_id,
            rando,
        }
    }

    async fn run_loop(&mut self) -> anyhow::Result<()> {
        let recv = self.global_chat.receiver().await;

        while let Some(received_message) = recv.next_message().await {
            let content = received_message.message;
            match content {
                GlobalChatMessageContent::MatchmakingMessage { ref msg } => {
                    match &msg {
                        MatchmakingMessage::LFG {
                            match_type,
                            rando: lfg_rando,
                        } => {
                            self.on_lfg_message(
                                match_type.clone(),
                                received_message.from,
                                *lfg_rando,
                            )
                            .await;
                        }
                        MatchmakingMessage::Handshake {
                            game_match,
                            handshake_type,
                            rando,
                        } => {
                            self.on_handshake_message(
                                game_match.clone(),
                                received_message.from,
                                handshake_type.clone(),
                                *rando,
                            )
                            .await;
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
        info!(
            "GameMatchmaker::on_lfg_message(rando = {:?}, my_rando = {:?})",
            lfg_rando, self.rando
        );
        if self.own_node_id == message_from {
            info!("GameMatchmaker::on_lfg_message: own_node_id == message_from -- ignore");
            return;
        }
        if lfg_match_type != self.match_type {
            info!("GameMatchmaker::on_lfg_message: lfg_match_type != self.match_type -- ignore");
            return;
        }
        if self.state.blacklist.contains(&message_from) {
            info!("GameMatchmaker::on_lfg_message: blacklist contains message_from -- ignore");
            return;
        }
        if self.state.result_opponent.is_some() {
            info!("GameMatchmaker::on_lfg_message: result_opponent is set -- ignore");
            return;
        }
        let new_match = GameMatch {
            match_id: uuid::Uuid::new_v4(),
            title: "".to_string(),
            type_: self.match_type.clone(),
            users: vec![self.own_node_id.clone(), message_from],
            seed: get_random_seed(),
            time: get_timestamp_now_ms(),
        };
        let reply = GlobalChatMessageContent::handshake_request(
            new_match,
            lfg_rando,
        );
        self.send_direct_message(message_from, reply).await;
        info!("send request");
    }
    
    async fn on_handshake_message(
        &mut self,
        game_match: GameMatch<NodeIdentity>,
        from: NodeIdentity,
        hand_type: MatchHandshakeType,
        hs_rando: MatchmakeRandomId,
    ) {
        warn!("GameMatchmaker::on_handshake_message(rando = {:?}, my_rando = {:?}, hand_type = {:?})", hs_rando, self.rando, hand_type);
        if self.own_node_id == from {
            info!("GameMatchmaker::on_handshake_message: own_node_id == from -- ignore");
            return;
        }
        if game_match.type_ != self.match_type {
            info!("GameMatchmaker::on_handshake_message: game_match.type_ != self.match_type -- ignore");
            return;
        }
        match hand_type {
            MatchHandshakeType::HandshakeRequest => {
                if self.state.blacklist.contains(&from) {
                    info!("GameMatchmaker::on_handshake_message: blacklist contains from -- ignore");
                    return;
                }
                if self.rando != hs_rando {
                    info!("GameMatchmaker::on_handshake_message: rando != hs_rando --  ignore");
                    return;
                }
                if self.state.game_match.is_some() {
                    info!("GameMatchmaker::on_handshake_message: game_match is set -- answer NO");
                    // answer NO
                    let no = GlobalChatMessageContent::handshake_no(
                        game_match, hs_rando,
                    );
                    self.send_direct_message(from, no).await;
                    info!("send no - done");
                    return;
                }
                info!("got good request - sending YES");
                let yes = GlobalChatMessageContent::handshake_yes(
                    game_match.clone(), hs_rando,
                );
                self.send_direct_message(from, yes).await;
                info!("send yes");
                self.on_start_pingpong(from, game_match).await;
            }
            MatchHandshakeType::AnswerYes => {
                if self.state.game_match.is_some() {
                    info!("GameMatchmaker::on_handshake_message: game_match is set -- ignore");
                    return;
                }
                info!("get yes");
                self.on_start_pingpong(from, game_match).await;
            }
            MatchHandshakeType::AnswerNo => {
                if self.state.game_match.is_some() {
                    info!("GameMatchmaker::on_handshake_message: game_match is set -- ignore");
                    return;
                }
                self.state.blacklist.insert(from);
                info!("get no");
            }
            MatchHandshakeType::Ping(ping) => {
                let Some(current_match) = self.state.game_match.as_ref() else {
                    info!("GameMatchmaker::on_handshake_message: game_match is not set -- ignore");
                    return;
                };
                if !current_match.users.contains(&from) {
                    info!("GameMatchmaker::on_handshake_message: game_match does not contain from -- ignore");
                    return;
                }
                if current_match.match_id != game_match.match_id {
                    info!("GameMatchmaker::on_handshake_message: game_match.match_id != current_match.match_id -- ignore");
                    return;
                }
                self.handle_ping(current_match.clone(), from, ping).await;
            }
        }
    }

    async fn handle_ping(&mut self, game_match: GameMatch<NodeIdentity>, from: NodeIdentity, ping: u8) {
        info!("GameMatchmaker::handle_ping: ping = {:?}", ping);
        let pong = GlobalChatMessageContent::MatchmakingMessage {
            msg: MatchmakingMessage::Handshake {
                game_match: game_match.clone(),
                handshake_type: MatchHandshakeType::Ping(ping + 1),
                rando: self.rando,
            },
        };
        self.send_direct_message(from, pong).await;
        if ping >= 10 {
            self.on_confirm_match(game_match).await;
        }
    }

    async fn on_confirm_match(&mut self, game_match: GameMatch<NodeIdentity>) {
        if let Err(e) = self.sender.send(game_match).await {
            warn!("on_confirm_match failed: {e}");
        }
    }

    async fn send_direct_message(
        &self,
        to: NodeIdentity,
        x: GlobalChatMessageContent,
    ) {
        if let Err(e) = self.global_chat.sender().direct_message(to, x).await {
            warn!("send_direct_message failed: {e}");
        }
    }

    async fn on_start_pingpong(&mut self, from: NodeIdentity, game_match: GameMatch<NodeIdentity>) {
        self.state.result_opponent = Some(from);
        self.state.game_match = Some(game_match.clone());

        let cc = self.global_chat.clone();
        let rando = self.rando;
        n0_future::task::spawn(async move {
            let sender = cc.sender();
            for _i in 0..3 {
                let pong = GlobalChatMessageContent::MatchmakingMessage {
                    msg: MatchmakingMessage::Handshake {
                        game_match: game_match.clone(),
                        handshake_type: MatchHandshakeType::Ping(0),
                        rando,
                    },
                };
                if let Err(e) = sender.direct_message(from, pong).await {
                    warn!("on_start_pingpong failed: {e}");
                };
                n0_future::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        });
    }
}

#[derive(Debug, Clone)]
struct GameMatchmakerState {
    game_match: Option<GameMatch<NodeIdentity>>,
    result_opponent: Option<NodeIdentity>,
    blacklist: HashSet<NodeIdentity>,
}

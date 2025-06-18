use std::sync::Arc;

use anyhow::Context;
use game::{
    api::game_match::GameMatch, futures_channel::mpsc::{unbounded, UnboundedReceiver}, futures_util::{lock::Mutex, FutureExt, Stream, StreamExt}, rule_manager::RuleManager, state_manager::GameStateManager, tet::{GameOverReason, GameState}
};
use protocol::{
    chat::{ChatController, IChatController, IChatReceiver, IChatSender},
    chat_ticket::ChatTicket,
    global_matchmaker::GlobalMatchmaker,
    user_identity::NodeIdentity,
    IChatRoomType, ReceivedMessage,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMessage {
    GameState(GameState),
    UserText(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game1v1RoomType;

impl IChatRoomType for Game1v1RoomType {
    type M = GameMessage;
    type P = ();
    fn default_presence() -> Self::P {
        ()
    }
}

#[derive(Debug)]
pub struct Game1v1MatchChatController {
    mm: GlobalMatchmaker,
    chat: ChatController<Game1v1RoomType>,
    match_info: GameMatch<NodeIdentity>,
    opponent_id: NodeIdentity,
}

pub async fn join_1v1_match(
    mm: GlobalMatchmaker,
    game_match: GameMatch<NodeIdentity>,
) -> anyhow::Result<Game1v1MatchChatController> {
    let ticket = ChatTicket::new_str_bs(
        &format!("1v1-{}", game_match.match_id),
        game_match
            .users
            .iter()
            .map(|m| m.node_id().clone())
            .collect(),
    );

    let node = mm.own_node().await.context("no node")?;
    tracing::info!("joining game: {:?}", ticket);
    let chat = node.join_chat::<Game1v1RoomType>(&ticket).await?;
    let opponent_id = game_match
        .users
        .iter()
        .find(|m| *m != node.node_identity())
        .context("no opponent")?
        .clone();
    Ok(Game1v1MatchChatController {
        mm,
        chat,
        opponent_id,
        match_info: game_match,
    })
}
use async_stream::stream;

impl Game1v1MatchChatController {
    pub async fn update_own_state(
        &self,
        next_state: GameState,
    ) -> anyhow::Result<()> {
        let sender = self.chat.sender();
        sender
            .broadcast_message(GameMessage::GameState(next_state))
            .await?;
        Ok(())
    }
    pub async fn send_text_msg(&self, msg: String) -> anyhow::Result<()> {
        let sender = self.chat.sender();
        sender.broadcast_message(GameMessage::UserText(msg)).await?;
        Ok(())
    }

    pub async fn opponent_move_stream(
        &self,
    ) -> impl Stream<Item = GameState> + Send + 'static {
        let opponent_id = self.opponent_id.clone();
        let receiver = self.chat.receiver().await;
        stream! {
            while let Some(ReceivedMessage {
                message: msg,
                from,
                ..
            }) = receiver.next_message().await {
                if from !=  opponent_id {
                    tracing::warn!("opponent move from wrong node: {:?}", from);
                    continue;
                }
                if let GameMessage::GameState(s) = msg {
                    // TODO: check from
                    tracing::info!("opponent move: {:?}: {:?}", from, s);
                    yield s;
                }
            }
        }
    }
    pub async fn opponent_message_stream(
        &self,
    ) -> impl Stream<Item = String> + Send + 'static {
        let opponent_id = self.opponent_id.clone();
        let receiver = self.chat.receiver().await;
        stream! {
            while let Some(ReceivedMessage {
                message: msg,
                from,
                ..
            }) = receiver.next_message().await {
                if from !=  opponent_id {
                    tracing::warn!("opponent message from wrong node: {:?}", from);
                    continue;
                }
                if let GameMessage::UserText(s) = msg {
                    yield s;
                }
            }
        }
    }
}


#[derive(Debug)]
pub struct Game1v1MatchOutcome {
    match_info: GameMatch<NodeIdentity>,
    winner: NodeIdentity,
    loser: NodeIdentity,
    reason: GameOverReason,
    winner_state: GameState,
    loser_state: GameState,
}


struct Game1v1StateManagerForPlayer(Game1v1MatchChatController);

#[async_trait::async_trait]
impl RuleManager for Game1v1StateManagerForPlayer {
    async fn accept_state(&self, state: GameState) 
    ->  anyhow::Result<Option<GameState>> {
        todo!()
    }
}

struct Game1v1StateManagerForSpectator(Mutex<UnboundedReceiver<GameState>>);
#[async_trait::async_trait]
impl RuleManager for Game1v1StateManagerForSpectator {
    async fn accept_state(&self, _state: GameState) 
    ->  anyhow::Result<Option<GameState>> {
        Ok( {self.0.lock().await.next().fuse()}.await)
    }
}


pub fn get_spectator_state_manager(cc: Game1v1MatchChatController, idx: i32) 
-> GameStateManager {
    let mut manager = GameStateManager::new(&cc.match_info.seed, cc.match_info.time);

    let (state_tx, state_rx) = unbounded();
    let spectate_rule = Game1v1StateManagerForSpectator(Mutex::new(state_rx));
    manager.add_rule(Arc::new(spectate_rule));
    

    manager
}


pub fn get_player_state_manager(cc:Game1v1MatchChatController)
 -> GameStateManager {
    let mut manager = GameStateManager::new(&cc.match_info.seed, cc.match_info.time);


    manager
}

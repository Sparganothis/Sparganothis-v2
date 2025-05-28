use anyhow::Context;
use game::{
    api::game_match::GameMatch,
    futures_util::{FutureExt, Stream, StreamExt},
    tet::{GameOverReason, GameState},
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
pub struct Game1v1MessageType;

impl IChatRoomType for Game1v1MessageType {
    type M = GameMessage;
    type P = ();
    fn default_presence() -> Self::P {
        ()
    }
}

#[derive(Debug)]
pub struct Game1v1MatchController {
    mm: GlobalMatchmaker,
    chat: ChatController<Game1v1MessageType>,
    match_info: GameMatch<NodeIdentity>,
    opponent_id: NodeIdentity,
}

pub async fn join_game(
    mm: GlobalMatchmaker,
    game_match: GameMatch<NodeIdentity>,
) -> anyhow::Result<Game1v1MatchController> {
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
    let chat = node.join_chat::<Game1v1MessageType>(&ticket).await?;
    let opponent_id = game_match
        .users
        .iter()
        .find(|m| *m != node.node_identity())
        .context("no opponent")?
        .clone();
    Ok(Game1v1MatchController {
        mm,
        chat,
        opponent_id,
        match_info: game_match,
    })
}
use async_stream::stream;

impl Game1v1MatchController {
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

pub struct Game1v1MatchOutcome {
    match_info: GameMatch<NodeIdentity>,
    winner: NodeIdentity,
    loser: NodeIdentity,
    reason: GameOverReason,
    winner_state: GameState,
    loser_state: GameState,
}

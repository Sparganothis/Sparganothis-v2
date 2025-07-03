use std::sync::Arc;

use anyhow::Context;
use game::futures_util::StreamExt;
use game::{
    api::game_match::GameMatch,
    futures_channel::mpsc::{unbounded, UnboundedReceiver},
    futures_util::{lock::Mutex, pin_mut, FutureExt, Stream},
    input::{
        callback_manager::InputCallbackManagerRule, events::GameInputEvent,
    },
    rule_manager::RuleManager,
    settings::GameSettings,
    state_manager::GameStateManager,
    tet::{GameOverReason, GameState},
};
use n0_future::{
    task::{spawn, AbortOnDropHandle},
    SinkExt,
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

#[derive(Debug, Clone)]
pub struct Game1v1MatchChatController {
    mm: GlobalMatchmaker,
    chat: ChatController<Game1v1RoomType>,
    match_info: GameMatch<NodeIdentity>,
    opponent_id: NodeIdentity,
}

impl PartialEq for Game1v1MatchChatController {
    fn eq(&self, other: &Self) -> bool {
        self.chat == other.chat
            && self.match_info == other.match_info
            && self.opponent_id == other.opponent_id
    }
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
use tokio::sync::RwLock;

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

// #[derive(Debug)]
// pub struct Game1v1MatchOutcome {
//     match_info: GameMatch<NodeIdentity>,
//     winner: NodeIdentity,
//     loser: NodeIdentity,
//     reason: GameOverReason,
//     winner_state: GameState,
//     loser_state: GameState,
// }

// struct Game1v1StateManagerForPlayer(Game1v1MatchChatController);

// #[async_trait::async_trait]
// impl RuleManager for Game1v1StateManagerForPlayer {
//     async fn accept_state(
//         &self,
//         state: GameState,
//     ) -> anyhow::Result<Option<GameState>> {
//         todo!()
//     }
// }

struct Game1v1SpectatorRule(Mutex<UnboundedReceiver<GameState>>);
#[async_trait::async_trait]
impl RuleManager for Game1v1SpectatorRule {
    async fn accept_state(
        &self,
        _state: GameState,
    ) -> anyhow::Result<Option<GameState>> {
        let r = Ok({ self.0.lock().await.next().fuse() }.await);
        r
    }
}

struct Game1v1YouWinIfOpponentLoseRule(Mutex<UnboundedReceiver<()>>);
#[async_trait::async_trait]
impl RuleManager for Game1v1YouWinIfOpponentLoseRule {
    /// if opponent lose, you win
    async fn accept_state(
        &self,
        mut _state: GameState,
    ) -> anyhow::Result<Option<GameState>> {
        tracing::info!("Game1v1YouWinIfOpponentLoseRule");
        if _state.game_over() {
            return Ok(None);
        }
        let _r = { self.0.lock().await.next().fuse() }.await;
        if _r.is_none() {
            anyhow::bail!("no actual message on recv");
        }
        _state.game_over_reason = Some(GameOverReason::Win);
        Ok(Some(_state))
    }
}

struct Game1v1RecvLinesFromOpponentRule(Mutex<UnboundedReceiver<GameState>>);
#[async_trait::async_trait]
impl RuleManager for Game1v1RecvLinesFromOpponentRule {
    async fn accept_state(
        &self,
        my_state: GameState,
    ) -> anyhow::Result<Option<GameState>> {
        let Some(opponent_state) = { self.0.lock().await.next().fuse() }.await
        else {
            anyhow::bail!("no oppoonent omves ???");
        };
        if my_state.game_over() || opponent_state.game_over() {
            return Ok(None);
        }
        if opponent_state.total_garbage_sent == my_state.garbage_recv {
            return Ok(None);
        }

        let mut new_state = my_state;
        new_state.apply_raw_received_garbage(opponent_state.total_garbage_sent);
        Ok(Some(new_state))
    }
}

pub fn get_spectator_state_manager(
    cc: Game1v1MatchChatController,
) -> GameStateManager {
    tracing::info!("get_spectator_state_manager()");
    let mut manager =
        GameStateManager::new(&cc.match_info.seed, cc.match_info.time);

    let (state_tx, state_rx) = unbounded();
    let spectate_rule = Game1v1SpectatorRule(Mutex::new(state_rx));
    manager.add_rule("spectate_rule", Arc::new(spectate_rule));

    manager.add_loop(async move {
        let oppstream = cc.opponent_move_stream().await;
        pin_mut!(oppstream);
        while let Some(state) = oppstream.next().await {
            state_tx.unbounded_send(state)?;
        }

        anyhow::Ok(())
    });

    manager
}

pub fn get_1v1_player_state_manager(
    cc: Game1v1MatchChatController,
    settings: Arc<RwLock<GameSettings>>,
    player_input: UnboundedReceiver<GameInputEvent>,
) -> GameStateManager {
    tracing::info!("get_1v1_player_state_manager");
    let mut game_state_manager =
        GameStateManager::new(&cc.match_info.seed, cc.match_info.time);

    let callback_manager = InputCallbackManagerRule::new(
        player_input,
        game_state_manager.read_state_stream(),
        settings,
    );
    game_state_manager.add_rule("callback_manager", Arc::new(callback_manager));

    let g2 = game_state_manager.clone();
    let cc2 = cc.clone();
    game_state_manager.add_loop(async move {
        let stream = g2.read_state_stream();
        pin_mut!(stream);
        while let Some(s) = stream.next().await {
            cc2.update_own_state(s).await?;
        }
        anyhow::Ok(())
    });

    let (tx_you_win, rx_you_win) = unbounded();
    let you_win_rule = Game1v1YouWinIfOpponentLoseRule(Mutex::new(rx_you_win));
    game_state_manager.add_rule("you_win_rule", Arc::new(you_win_rule));

    let (state_tx2, state_rx2) = unbounded();
    let send_line_rule =
        Game1v1RecvLinesFromOpponentRule(Mutex::new(state_rx2));
    game_state_manager.add_rule("sendline", Arc::new(send_line_rule));

    let cc2 = cc.clone();
    game_state_manager.add_loop(async move {
        let stream = cc2.opponent_move_stream().await;
        pin_mut!(stream);
        while let Some(oppponent_state) = stream.next().await {
            tracing::info!("\n\n NEW opponent state receive!!!");
            if state_tx2.unbounded_send(oppponent_state).is_err() {
                tracing::info!("failed to notify send lines function");
            }
            if oppponent_state.game_over() {
                let _e = tx_you_win.unbounded_send(());
                if _e.is_err() {
                    tracing::info!(
                        "failed to notify opponent lost game: {_e:#?}"
                    );
                }
            }
        }
        anyhow::Ok(())
    });

    game_state_manager
}

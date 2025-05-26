use anyhow::Context;
use game::{api::game_match::GameMatch, tet::GameState};
use protocol::{chat::ChatController, chat_ticket::ChatTicket, global_matchmaker::GlobalMatchmaker, user_identity::NodeIdentity, IChatRoomType};
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
    chat: ChatController<Game1v1MessageType>
}


pub async fn join_game(mm: GlobalMatchmaker, game_match: GameMatch<NodeIdentity>)
 -> anyhow::Result<Game1v1MatchController> {
    let ticket = ChatTicket::new_str_bs(
        &format!("1v1-{}", game_match.match_id), 
        game_match.users.iter().map(
            |m| m.node_id().clone()
        ).collect());
    
    let node = mm.own_node().await.context("no node")?;
    let chat =
        node.join_chat::<Game1v1MessageType>(&ticket).await?;
    Ok(Game1v1MatchController {
        chat
    })
}

impl Game1v1MatchController {
    pub fn update_own_state(next_state: GameState) {
        todo!()
    }
    pub fn send_text_msg(msg: String) {
        todo!()
    }

    pub fn opponent_move_stream() {
        todo!()
    }
    pub fn opponent_message_stream() {
        todo!()
    }
    
    pub fn game_over_notify() {
        todo!()
    }
    pub fn countdown_notify() {
        todo!()
    }
    pub fn game_start_notify() {
        todo!()
    }
}
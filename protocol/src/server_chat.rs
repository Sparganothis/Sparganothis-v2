use game::{api::game_match::{GameMatch, GameMatchType}, tet::GameSeed};
use serde::{Deserialize, Serialize};

use crate::{
    chat_ticket::ChatTicket, game_matchmaker::MatchmakeRandomId,
    user_identity::NodeIdentity, IChatRoomType,
};


#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ServerChatRoomType;

impl IChatRoomType for ServerChatRoomType {
    type M = ServerChatMessageContent;
    type P = ServerChatPresence;
    fn default_presence() -> Self::P {
        ServerChatPresence::default()
    }
}
#[derive(
    Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize, Default,
)]
pub struct ServerChatPresence {
    pub is_server: bool,
}

#[derive(
    Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub enum ServerChatMessageContent {
    Request(ServerMessageRequest),
    Reply(Result<ServerMessageReply, String>)
}

#[derive(
    Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub enum ServerMessageRequest {
    GuestLoginMessage {},
}

#[derive(
    Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub enum ServerMessageReply {
    GuestLoginMessage {},
}
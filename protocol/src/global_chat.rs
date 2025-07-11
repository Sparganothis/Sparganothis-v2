use game::api::game_match::{GameMatch, GameMatchType};
use serde::{Deserialize, Serialize};

use crate::{
    chat_ticket::ChatTicket, server_chat_api::api_method_macros::ServerInfo,
    IChatRoomType,
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GlobalChatRoomType;

impl IChatRoomType for GlobalChatRoomType {
    type M = GlobalChatMessageContent;
    type P = GlobalChatPresence;
    fn default_presence() -> Self::P {
        GlobalChatPresence::default()
    }
}
#[derive(
    Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize, Default,
)]
pub struct GlobalChatPresence {
    pub url: String,
    pub platform: String,
    pub is_server: Option<ServerInfo>,
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum GlobalChatMessageContent {
    TextMessage {
        text: String,
    },
    // MatchmakingMessage {
    //     msg: MatchmakingMessage,
    // },
    SpectateMatch {
        ticket: ChatTicket,
        match_type: String,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MatchHandshakeType {
    HandshakeRequest,
    AnswerYes,
    AnswerNo,
    Ping(u8),
}

impl From<String> for GlobalChatMessageContent {
    fn from(value: String) -> Self {
        Self::TextMessage { text: value }
    }
}

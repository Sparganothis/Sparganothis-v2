use serde::{Deserialize, Serialize};

use crate::{
    api::api_method_macros::ServerInfo, chat::chat_presence::PresenceList,
    chat::chat_ticket::ChatTicket, IChatRoomType,
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    BootstrapQuery(GlobalChatBootstrapQuery),
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GlobalChatBootstrapQuery {
    PlzSendServerList,
    ServerList { v: PresenceList<GlobalChatPresence> },
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

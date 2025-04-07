use game::api::game_match::GameMatchType;
use serde::{Deserialize, Serialize};

use crate::{chat_ticket::ChatTicket, IChatRoomType};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct GlobalChatMessageType;

impl IChatRoomType for GlobalChatMessageType {
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
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum GlobalChatMessageContent {
    TextMessage {
        text: String,
    },
    MatchmakingMessage {
        msg: MatchmakingMessage,
    },
    SpectateMatch {
        ticket: ChatTicket,
        match_type: String,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MatchmakingMessage {
    LFG {
        match_type: GameMatchType,
    },
    Handshake {
        match_type: GameMatchType,
        handshake_type: MatchHandshakeType,
    },
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MatchHandshakeType {
    HandshakeRequest,
    AnswerYes,
    AnswerNo,
}

impl From<String> for GlobalChatMessageContent {
    fn from(value: String) -> Self {
        Self::TextMessage { text: value }
    }
}

impl GlobalChatMessageContent {
    pub fn handshake_request(match_type: GameMatchType) -> Self {
        Self::MatchmakingMessage {
            msg: MatchmakingMessage::Handshake {
                match_type,
                handshake_type: MatchHandshakeType::HandshakeRequest,
            },
        }
    }
    pub fn handshake_yes(match_type: GameMatchType) -> Self {
        Self::MatchmakingMessage {
            msg: MatchmakingMessage::Handshake {
                match_type,
                handshake_type: MatchHandshakeType::AnswerYes,
            },
        }
    }
    pub fn handshake_no(match_type: GameMatchType) -> Self {
        Self::MatchmakingMessage {
            msg: MatchmakingMessage::Handshake {
                match_type,
                handshake_type: MatchHandshakeType::AnswerNo,
            },
        }
    }
    pub fn matchmake_lfg(match_type: GameMatchType) -> Self {
        Self::MatchmakingMessage {
            msg: MatchmakingMessage::LFG {
                match_type,
            },
        }
    }
}

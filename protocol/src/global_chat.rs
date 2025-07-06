use game::api::game_match::{GameMatch, GameMatchType};
use serde::{Deserialize, Serialize};

use crate::{
    chat_ticket::ChatTicket, game_matchmaker::MatchmakeRandomId, server_chat_api::server_info::ServerInfo, user_identity::NodeIdentity, IChatRoomType
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
        rando: MatchmakeRandomId,
    },
    Handshake {
        game_match: GameMatch<NodeIdentity>,
        handshake_type: MatchHandshakeType,
        rando: MatchmakeRandomId,
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

impl GlobalChatMessageContent {
    pub fn handshake_request(
        game_match: GameMatch<NodeIdentity>,
        rando: MatchmakeRandomId,
    ) -> Self {
        Self::MatchmakingMessage {
            msg: MatchmakingMessage::Handshake {
                game_match,
                handshake_type: MatchHandshakeType::HandshakeRequest,
                rando,
            },
        }
    }
    pub fn handshake_yes(
        game_match: GameMatch<NodeIdentity>,
        rando: MatchmakeRandomId,
    ) -> Self {
        Self::MatchmakingMessage {
            msg: MatchmakingMessage::Handshake {
                game_match,
                handshake_type: MatchHandshakeType::AnswerYes,
                rando,
            },
        }
    }
    pub fn handshake_no(
        game_match: GameMatch<NodeIdentity>,
        rando: MatchmakeRandomId,
    ) -> Self {
        Self::MatchmakingMessage {
            msg: MatchmakingMessage::Handshake {
                game_match,
                handshake_type: MatchHandshakeType::AnswerNo,
                rando,
            },
        }
    }
    pub fn matchmake_lfg(
        match_type: GameMatchType,
        rando: MatchmakeRandomId,
    ) -> Self {
        Self::MatchmakingMessage {
            msg: MatchmakingMessage::LFG { match_type, rando },
        }
    }
}

use serde::{Deserialize, Serialize};

use crate::IChatRoomType;

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
#[derive(
    Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize,
)]
pub enum GlobalChatMessageContent {
    TextMessage {
        text: String
    },
    MatchmakingMessage {
        _1v1_options: Option<uuid::Uuid>,
    }
}

impl From<String> for GlobalChatMessageContent {
    fn from(value: String) -> Self {
        Self::TextMessage { text: value }
    }
}
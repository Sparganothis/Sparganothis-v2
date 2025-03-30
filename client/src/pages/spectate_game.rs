use std::collections::BTreeSet;

use crate::{
    comp::game_display::GameDisplay, network::NetworkState, pages::ChatRoom,
};
use dioxus::prelude::*;
use game::tet::GameState;
use iroh::NodeId;
use protocol::{chat_ticket::ChatTicket, IChatRoomType as ChatMessageType2};
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::{FromUserInput, RenderElement};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameMessageSpam;

impl ChatMessageType2 for GameMessageSpam {
    type M = GameMessage;
    type P = ();
    fn default_presence() -> Self::P {
        ()
    }
}

impl FromUserInput for GameMessageSpam {
    fn from_user_input(input: String) -> Self::M {
        GameMessage::UserText(input)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMessage {
    GameState(GameState),
    UserText(String),
}

impl RenderElement for GameMessageSpam {
    fn render_message(message: <Self as ChatMessageType2>::M) -> Element {
        match message {
            GameMessage::GameState(game_state) => {
                rsx! {
                    div {
                        style:"height: 500px; min-height: 500px;",
                        GameDisplay { game_state }
                    }
                }
            }
            GameMessage::UserText(text) => {
                rsx! {
                    {text}
                }
            }
        }
    }

    fn render_presence(_payload: Option<<Self as ChatMessageType2>::P>) -> Element {
        rsx! {
            br{}
        }
    }
}

#[component]
pub fn SpectateGamePage(node_id: NodeId) -> Element {
    let mm = use_context::<NetworkState>().global_mm;
    let presence = use_signal(|| vec![]);
    let chat = use_resource(move || {
        let mm = mm.read().clone();
        async move {
            let Some(mm) = mm else {
                return None;
            };
            let Some(nn) = mm.own_node().await else {
                return None;
            };
            let chat_ticket =
                ChatTicket::new_str_bs("play", BTreeSet::from([node_id]));
            let Ok(chat) = nn.join_chat::<GameMessageSpam>(&chat_ticket).await
            else {
                warn!("Failed to join chat");
                return None;
            };
            Some(chat)
        }
    });
    let chat =
        use_memo(move || chat.read().as_ref().map(|c| c.clone()).flatten());
    rsx! {
        ChatRoom<GameMessageSpam> { chat, presence }
    }
}

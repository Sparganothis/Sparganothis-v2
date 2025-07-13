use std::collections::BTreeSet;

use crate::{comp::{
    chat::{
        chat_signals_hook::use_chat_signals,
        chat_traits::{FromUserInput, RenderElement},
        chat_window_fullscreen::FullscreenChatRoom,
    },
    game_display::GameDisplay,
}, route::{Route, UrlParam}};
use dioxus::prelude::*;
use game::{api::game_match::GameMatch, tet::GameState};
use iroh::NodeId;
use protocol::{
    chat_ticket::ChatTicket, global_matchmaker::GlobalMatchmaker, user_identity::NodeIdentity, IChatRoomType as ChatMessageType2
};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateLobyRoomType;

impl ChatMessageType2 for PrivateLobyRoomType {
    type M = PrivateLobbyMessage;
    type P = bool;
    fn default_presence() -> Self::P {
        false
    }
}

impl FromUserInput for PrivateLobyRoomType {
    fn from_user_input(input: String) -> Self::M {
        PrivateLobbyMessage::UserText(input)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivateLobbyMessage {
    StartPlaying(GameMatch<NodeIdentity>),
    UserText(String),
}

impl RenderElement for PrivateLobyRoomType {
    fn render_message(message: <Self as ChatMessageType2>::M) -> Element {
        match message {
            PrivateLobbyMessage::StartPlaying(game_state) => {
                let url = Route::Play1v1Page { game_match: UrlParam(game_state) };
                navigator().push(url.clone());
                rsx! {
                    h1 {
                        a {
                            href :  "{url.to_string()}",
                            "GO TO GAME!"
                        }
                    }
                }
            }
            PrivateLobbyMessage::UserText(text) => {
                rsx! {
                    {text}
                }
            }
        }
    }

    fn render_presence(
        _payload: Option<<Self as ChatMessageType2>::P>,
    ) -> Element {
        rsx! {
            br{}
        }
    }
}

#[component]
pub fn PrivateLobbyChatBox(owner_id: NodeIdentity, room_uuid: uuid::Uuid) -> Element {
    let chat = use_chat_signals(
        true,
        Callback::new(move |mm: GlobalMatchmaker| async move {
            let Some(nn) = mm.own_node().await else {
                return None;
            };
            let chat_ticket = &format!("{room_uuid}")[..30];
            let chat_ticket =
                ChatTicket::new_str_bs(&chat_ticket, BTreeSet::from([owner_id.node_id().clone()]));
            let Ok(chat) = nn.join_chat::<PrivateLobyRoomType>(&chat_ticket).await
            else {
                warn!("Failed to join chat");
                return None;
            };
            Some(chat)
        }),
    );

    rsx! {
        FullscreenChatRoom<PrivateLobyRoomType> { chat }
    }
}

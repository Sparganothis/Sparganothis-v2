use std::collections::VecDeque;

use dioxus::prelude::*;
use protocol::{
    chat::{ChatController, IChatController, IChatSender},
    datetime_now,
    global_matchmaker::{
        GlobalChatMessageType, GlobalChatPresence, GlobalMatchmaker,
    },
    IChatRoomType as ChatMessageType2, ReceivedMessage,
};
use tracing::warn;

pub trait ChatMessageType:
    ChatMessageType2 + RenderElement + FromUserInput + Clone
{
}
impl<T> ChatMessageType for T where
    T: ChatMessageType2 + RenderElement + FromUserInput + Clone
{
}
pub trait RenderElement: ChatMessageType2 {
    fn render_message(message: <Self as ChatMessageType2>::M) -> Element;
    fn render_presence(
        payload: Option<<Self as ChatMessageType2>::P>,
    ) -> Element;
}
pub trait FromUserInput: ChatMessageType2 {
    fn from_user_input(input: String) -> <Self as ChatMessageType2>::M;
}

impl FromUserInput for GlobalChatMessageType {
    fn from_user_input(input: String) -> <Self as ChatMessageType2>::M {
        input
    }
}
impl RenderElement for GlobalChatMessageType {
    fn render_message(message: <Self as ChatMessageType2>::M) -> Element {
        rsx! {
                "{message}"
        }
    }
    fn render_presence(payload: Option<GlobalChatPresence>) -> Element {
        match payload {
            Some(payload) => {
                rsx! {
                    br{}
                    if ! payload.platform.is_empty() {
                        small {
                            "{payload.platform}:"
                        }
                    }
                    if ! payload.url.is_empty() {
                        small {
                            "{payload.url}"
                        }
                    }
                }
            }
            None => rsx! {
                br{}
            },
        }
    }
}

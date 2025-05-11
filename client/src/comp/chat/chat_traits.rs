use dioxus::prelude::*;
use protocol::{
    global_chat::{
        GlobalChatMessageContent, GlobalChatMessageType, GlobalChatPresence,
    },
    IChatRoomType as ChatMessageType2,
};

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
        GlobalChatMessageContent::TextMessage { text: input }
    }
}
impl RenderElement for GlobalChatMessageType {
    fn render_message(message: <Self as ChatMessageType2>::M) -> Element {
        let message = match message {
            GlobalChatMessageContent::TextMessage { text } => text,
            _x => {
                format!("{:#?}", _x)
            }
        };
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
                            "{truncate_str(payload.platform.clone())}:"
                        }
                    }
                    if ! payload.url.is_empty() {
                        small {
                            "{truncate_str(payload.url.clone())}"
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

fn truncate_str(s: String) -> String {
    if s.len() < 20 {
        return s;
    }
    s[0..20].to_string()
}

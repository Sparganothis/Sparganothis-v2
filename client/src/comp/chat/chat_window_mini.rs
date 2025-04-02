use crate::comp::{
    chat::{
        chat_display::{ChatHistoryDisplay, ChatPresenceDisplay},
        chat_input::ChatInput,
        chat_signals_hook::{
            use_chat_history_signal, use_chat_message_callback,
            use_chat_presence_signal, use_global_chat_controller_signal,
        },
    },
    icon::Icon,
};
use dioxus::prelude::*;
use protocol::chat_presence::PresenceList;

use super::{
    chat_signals_hook::{ChatControllerSignal, ChatHistory},
    chat_traits::ChatMessageType,
};

#[component]
pub fn MiniChatOverlay() -> Element {
    let chat = use_global_chat_controller_signal();

    rsx! {
        article {
            id: "mini_chat_overlay",
            style: "
            position: absolute;
            right: 1.5rem;
            bottom: 1.5rem;
            padding: 0.5rem;
            margin: 0.5rem;
            width: 420px;
            height: 666px;
            // border: 3px dashed blue;
            z-index: 2;
            background-color: white;
            ",
            MiniChatRoom {
                chat
            }
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MiniChatTabSelection {
    Chat,
    UserList,
    Minified,
}

#[component]
pub fn MiniChatRoom<T: ChatMessageType>(
    chat: ChatControllerSignal<T>,
) -> Element {
    let tabs_select = use_signal(move || MiniChatTabSelection::Chat);
    let presence = use_chat_presence_signal(chat);
    let history = use_chat_history_signal(chat);
    let on_user_message = use_chat_message_callback(chat, Some(history));

    rsx! {
        div {
            style: r#"
            display: grid; 
            grid-template-columns: 1fr; 
            grid-template-rows: 0.2fr 1.9fr 0.3fr; 
            gap: 0px 0px; 
            grid-template-areas: "topbar"   "mainchat"  "tabs"; 
            width: 100%;
            height: 100%;
            "#,

            div {
                style: "
                grid-area: topbar;
                width: 100%;
                height: 100%;
                // border: 1px solid green;
                container-type: size;
                ",

                MiniChatTopBar {selected: tabs_select}
            }
            div {
                style: "
                grid-area: mainchat;
                width: 100%;
                height: 100%;
                // border: 1px solid red;
                container-type: size;
                overflow-y: scroll;
                overflow-x: hidden;
                ",

                MiniChatContent::<T> {selected: tabs_select, presence, history}
            }
            div {
                style: "
                grid-area: tabs;
                width: 100%;
                height: 100%;
                // border: 1px solid blue;
                container-type: size;
                margin-top: -30px;
                ",
                ChatInput::<T> { on_user_message }
            }
        }
    }
}

#[component]
fn MiniChatTopBar(selected: Signal<MiniChatTabSelection>) -> Element {
    let chat_selected =
        use_memo(move || *selected.read() == MiniChatTabSelection::Chat);
    let user_list_selected =
        use_memo(move || *selected.read() == MiniChatTabSelection::UserList);
    use dioxus_free_icons::icons::bs_icons::*;
    let click_chat = Callback::new(move |_| {
        selected.set(MiniChatTabSelection::Chat);
    });
    let click_userlist = Callback::new(move |_| {
        selected.set(MiniChatTabSelection::UserList);
    });
    let click_x = Callback::new(move |_| {
        selected.set(MiniChatTabSelection::Minified);
    });

    rsx! {
        div {
            style: "
            display: flex;
            justify-content: center;
            align-items: center;
            ",

            Icon {
                icon:  BsChatRightText,
                color: "green",
                selected:  *chat_selected.read(),
                onclick: click_chat,
                tooltip: "Chat".to_string(),
            },
            Icon {
                icon: BsPersonLinesFill,
                color: "blue",
                selected: *user_list_selected.read(),
                onclick: click_userlist,
                tooltip: "User List".to_string(),
            },
            Icon {
                icon:  BsXLg,
                color: "red",
                selected: false,
                onclick: click_x,
                tooltip: "Close".to_string(),
            }
        }
    }
}

#[component]
fn MiniChatContent<T: ChatMessageType>(
    selected: Signal<MiniChatTabSelection>,
    presence: ReadOnlySignal<PresenceList<T>>,
    history: ReadOnlySignal<ChatHistory<T>>,
) -> Element {
    match *selected.read() {
        MiniChatTabSelection::Minified => rsx! {
            h1 {
                "X"
            }
        },
        MiniChatTabSelection::Chat => rsx! {
            ChatHistoryDisplay::<T> { history }

        },
        MiniChatTabSelection::UserList => rsx! {
            ChatPresenceDisplay::<T> { presence }
        },
    }
}

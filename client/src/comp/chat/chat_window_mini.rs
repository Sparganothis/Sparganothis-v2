use super::{
    chat_signals_hook::{ChatHistory, ChatSignals},
    chat_traits::ChatMessageType,
};
use crate::{
    comp::{
        chat::{
            chat_display::{ChatHistoryDisplay, ChatPresenceDisplay},
            chat_input::ChatInput,
        },
        icon::Icon,
    },
    localstorage::LocalStorageContext,
};
use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::BsMessenger;
use protocol::chat_presence::PresenceList;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(
    Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub enum MiniChatTabSelection {
    Chat,
    UserList,
    Minified,
}

#[component]
pub fn MiniChatRoomOverlay<T: ChatMessageType>(
    chat: ChatSignals<T>,
) -> Element {
    info!("MiniChatRoomOverlay");
    let mut tabs_select =
        use_context::<LocalStorageContext>().session.tab_select;
    let hide =
        use_memo(move || *tabs_select.read() == MiniChatTabSelection::Minified);

    let presence = chat.presence;
    let history = chat.history;
    let on_user_message = chat.send_broadcast_user_message;

    rsx! {
        if *hide.read() {
            MiniChatRoomOverlayButton {
                onclick: Callback::new(move |_| {
                    tabs_select.set(MiniChatTabSelection::Chat);
                }),
            }
        } else {
            MiniChatOverlayContainer {
                MiniChatImpl {
                    tabs_select,
                    presence,
                    history,
                    on_user_message,
                }
            }
        }
    }
}

#[component]
fn MiniChatOverlayContainer(children: Element) -> Element {
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
            height: calc(min(666px, 88%));
            // border: 3px dashed blue;
            z-index: 2;
            background-color: white;
            ",
            small {
                {children}
            }
        }
    }
}

#[component]
fn MiniChatRoomOverlayButton(onclick: Callback<()>) -> Element {
    rsx! {
        div {
            style: "
            display:pointer;
            position: absolute;
            right: 1em;
            bottom: 1em;
            ",
            onclick: move |_| onclick.call(()),
            button {
                class: "secondary outline",
                Icon {
                    icon: BsMessenger,
                    color: "blue",
                    selected: false,
                    onclick: Callback::new(move |_| onclick.call(())),
                    tooltip: "Open Chat".to_string(),
                }
            }
        }
    }
}

#[component]
fn MiniChatImpl<T: ChatMessageType>(
    tabs_select: Signal<MiniChatTabSelection>,
    presence: ReadOnlySignal<PresenceList<T>>,
    history: ReadOnlySignal<ChatHistory<T>>,
    on_user_message: Callback<T::M>,
) -> Element {
    rsx! {
        div {
            style: r#"
            display: grid; 
            grid-template-columns: 1fr; 
            grid-template-rows: 0.2fr 1.9fr 0.25fr; 
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
                MiniChatFooter::<T> {selected: tabs_select, on_user_message}
            }
        }
    }
}

#[component]
fn MiniChatFooter<T: ChatMessageType>(
    selected: Signal<MiniChatTabSelection>,
    on_user_message: Callback<T::M>,
) -> Element {
    match *selected.read() {
        MiniChatTabSelection::Minified => rsx! {
            h1 { "X"   }
        },
        MiniChatTabSelection::Chat => rsx! {
            ChatInput::<T> { on_user_message }
        },
        MiniChatTabSelection::UserList => rsx! {
            h1 {
                "User List"
            }
        },
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

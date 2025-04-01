use dioxus::prelude::*;
use protocol::{chat::{ChatController, IChatController, IChatReceiver}, chat_presence::PresenceList, global_matchmaker::GlobalChatMessageType, IChatRoomType, ReceivedMessage};
use tracing::warn;

use crate::{comp::{chat_comp::{chat_send_message, ChatHistory, ChatHistoryDisplay, ChatInput, ChatPresenceDisplay}, icon::Icon}, network::NetworkState};

use super::chat_comp::ChatMessageType;

#[component]
pub fn MiniChatOverlay () -> Element {

    let mm = use_context::<NetworkState>().global_mm;
    let chat = use_resource(move || {
        let mm = mm.read().clone();
        async move { Some(mm?.global_chat_controller().await?) }
    });
    let chat =
        use_memo(move || chat.read().as_ref().map(|c| c.clone()).flatten());
    let presence = use_context::<NetworkState>().global_presence_list;

    let mut history = use_signal(ChatHistory::<GlobalChatMessageType>::default);

    let _ = use_resource(move || {
        let mm = mm.read().clone();
        let chat = chat.read().clone();
        async move {
            let Some(_mm) = mm else {
                return;
            };
            let Some(controller) = chat else {
                return;
            };
            let recv = controller.receiver().await;
            while let Some(message) = recv.next_message().await {
                history.write().push(Ok(message));
            }
            warn!("XXX: ChatRoom receiver stream closed");
        }
    });

    use_effect(move || {
        let _i2 = use_context::<NetworkState>().is_connected.read().clone();
        *history.write() = ChatHistory::<GlobalChatMessageType>::default();
    });
    let on_user_message = Callback::new(move |message: <GlobalChatMessageType as IChatRoomType>::M| {
        let m = chat_send_message(mm.clone(), chat.into(), message);
        if let Some(m) = &m {
            history.write().push(Ok(m.clone()));
        } else {
            history
                .write()
                .push(Err("Failed to send message".to_string()));
        }
        m
    });

    rsx! {
        article {
            id: "mini_chat_overlay",
            style: "
            position: absolute;
            right: 1.5rem;
            bottom: 1.5rem;
            padding: 0.5rem;
            margin: 0.5rem;
            width: 350px;
            height: 450px;
            // border: 3px dashed blue;
            z-index: 2;
            background-color: white;
            ",
            MiniChatRoom {
                chat,
                presence,
                on_user_message,
                history,
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
pub fn MiniChatRoom<T: ChatMessageType> (
    chat: ReadOnlySignal<Option<ChatController<T>>>,
    presence: ReadOnlySignal<PresenceList<T>>,
    on_user_message: Callback<T::M, Option<ReceivedMessage<T>>>,
    
    history: ReadOnlySignal<ChatHistory<T>>,
) -> Element {
    let tabs_select = use_signal(move || MiniChatTabSelection::Chat);

    rsx! {
        div {
            style: r#"
            display: grid; 
            grid-template-columns: 1fr; 
            grid-template-rows: 0.3fr 1.9fr 0.3fr; 
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
                ",
                ChatInput::<T> { on_user_message }
            }
        }
    }
}


#[component]
fn MiniChatTopBar (selected: Signal<MiniChatTabSelection>) -> Element {
    let chat_selected=  use_memo(move || *selected.read() == MiniChatTabSelection::Chat);
    let user_list_selected= use_memo(move || *selected.read() == MiniChatTabSelection::UserList);
    use dioxus_free_icons::icons::bs_icons::*;
    let click_chat = Callback::new( move |_| {
        selected.set(MiniChatTabSelection::Chat);
    });
    let click_userlist = Callback::new( move |_|{
        selected.set(MiniChatTabSelection::UserList);
    });
    let click_x = Callback::new( move |_|{
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
            },
            Icon {
                icon: BsPersonLinesFill,
                color: "blue",
                selected: *user_list_selected.read(),
                onclick: click_userlist,
            },
            Icon {
                icon:  BsXLg,
                color: "red",
                selected: false,
                onclick: click_x,
            }
        }
    }
}

#[component]
fn MiniChatContent<T: ChatMessageType> (
    selected: Signal<MiniChatTabSelection>, 
    presence: ReadOnlySignal<PresenceList<T>>,
    history: ReadOnlySignal<ChatHistory<T>>,
) -> Element {
    match *selected.read() {
        MiniChatTabSelection::Minified => rsx!{
            h1 {
                "X"
            }
        },
        MiniChatTabSelection::Chat => rsx! {
            ChatHistoryDisplay::<T> { history }
            
        },
        MiniChatTabSelection::UserList => rsx! {
            ChatPresenceDisplay::<T> { presence }
        }
    }
}

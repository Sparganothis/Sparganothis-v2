#![allow(deprecated)]

use std::{collections::BTreeSet, time::Duration};

use crate::{
    comp::chat::{chat_signals_hook::ChatSignals, private_lobby_chat::{PrivateLobbyChatBox, PrivateLobbyMessage, PrivateLobyRoomType}},
    network::NetworkState,
    route::{Route, UrlParam},
};
use dioxus::prelude::*;
use game::{api::game_match::GameMatch, tet::get_random_seed, timestamp::get_timestamp_now_ms};
use protocol::{chat::{IChatController, IChatReceiver, IChatSender}, user_identity::NodeIdentity};
use tracing::info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    pub type WindowLocation;

    #[wasm_bindgen]
    pub type Window;

    #[wasm_bindgen(js_name = "window")]
    pub static WINDOW: Window;

    #[wasm_bindgen(method, getter, js_name = "location")]
    pub fn get_location(this: &Window) -> WindowLocation;

    #[wasm_bindgen(method, getter, js_name = "href")]
    pub fn get_href(this: &WindowLocation) -> JsValue;
}

#[wasm_bindgen]
pub fn retrieve_href() -> JsValue {
    WINDOW.get_location().get_href()
}

#[component]
pub fn PrivateLobbyPage(
    owner_id: ReadOnlySignal<UrlParam<NodeIdentity>>,
    room_uuid: ReadOnlySignal<uuid::Uuid>,
) -> Element {
    let url = use_memo(move || {
        let owner_id = owner_id.read().clone();
        let room_uuid = room_uuid.read().clone();
        let _u1 = Route::PrivateLobbyPage {
            room_uuid,
            owner_id,
        }
        .to_string();
        let u2 = retrieve_href().as_string();
        u2
    });
    let owner_id = use_memo(move || owner_id.read().0.clone());
    let mut copied = use_signal(move || false);

    rsx! {
        div {
            class:"container",


            article {
                PrivateLobbyChatBox {owner_id: owner_id.read().clone(), room_uuid: room_uuid.read().clone(),



                    h1 {
                        "Private 1v1 Room"
                        br {}
                        "owner: {owner_id.read().nickname()}"
                        br {}
                    }
                    article {
                        style: "width: 100%; display: flex; flex-direction:row height: 80px;",
                        if let Some(u2) = url.read().clone() {
                            a {
                                style: "width: 400px; height: 60px;",
                                href: "{u2}",
                                onclick: move |_x| {
                                    _x.prevent_default();
                                    write_to_clipboard(url.read().clone().unwrap_or(String::new()));
                                    copied.set(true);
                                },
                                "CLICKME TO COPY THE LINK",
                                br {},
                                "{room_uuid.read().clone():?}"
                            }
                            div {
                                style: "width: 400px; height: 60px;",
                                if *copied.read() {
                                    "📋  COPIED LINK TO CLIPBOARD! 👍 "
                                }
                            }
                        }
                    }
                    RoomControlComponent {owner_id: owner_id.read().clone(), room_uuid: room_uuid.read().clone()}



                }
            }
        }
    }
}

fn write_to_clipboard(string: String) {
    use wasm_bindgen_futures::spawn_local;
    let _task = spawn_local(async move {
        let window = web_sys::window().expect("window"); // { obj: val };
        let a = window.navigator().clipboard();
        let p = a.write_text(&string);
        let _result = wasm_bindgen_futures::JsFuture::from(p)
            .await
            .expect("clipboard populated");
        info!("clippyboy worked");
    });
}

#[component]
fn RoomControlComponent(
    owner_id: ReadOnlySignal<NodeIdentity>,
    room_uuid: ReadOnlySignal<uuid::Uuid>,
) -> Element {
    let mm = use_context::<NetworkState>().global_mm;
    let Some(mm) = mm.read().clone() else {
        return rsx! {
            "loading"
        };
    };
    let our_id = mm.own_node_identity();
    let chat = use_context::< ChatSignals<PrivateLobyRoomType> >();
    

    rsx! {

        article {
            style:"padding:10px;margin:10px;",

            if our_id == owner_id.read().clone() {
                RoomControlAdmin{our_id, owner_id, room_uuid, chat}
            } else {
                RoomControlPlayer{our_id, owner_id, room_uuid, chat}
            }
        }
    }
}

#[component]
fn RoomControlAdmin(
    our_id: ReadOnlySignal<NodeIdentity>,
    owner_id: ReadOnlySignal<NodeIdentity>,
    room_uuid: ReadOnlySignal<uuid::Uuid>,
    chat:  ChatSignals<PrivateLobyRoomType> ,
) -> Element {
    let mut ready_users = use_signal(BTreeSet::new);

    let chat = chat.chat.read().clone();
    let Some(chat) = chat else {return rsx!{"no chat"}};
    let chat2 = chat.clone();
    let _r = use_resource(move || {
        let chat2 = chat2.clone();
        async move {
            let rx = chat2.receiver().await;
            while let Some(msg) = rx.next_message().await {
                let _from = msg.from;
                let msg_content = &msg.message;
                let PrivateLobbyMessage::ReadyToPlay(ready) = msg_content else {
                    continue;
                };
                let ready = *ready;
                if ready {
                    ready_users.write().insert(_from);
                } else {
                    ready_users.write().remove(&_from);
                }
            }
        }
    });
    let ready_users = use_memo(move || ready_users.read().clone());
    let enable_start_button = use_memo(move || {
        let len = ready_users.read().len();
        len == 1
    });

    let chat3_send = chat.sender();
    let mm = use_context::<NetworkState>().global_mm;
    let mm = mm.read().clone();
    let Some(mm) = mm else {return rsx!{"loading"}};

    rsx! {
        h1 {"Admin"}
        button {
            disabled: !*enable_start_button.read(),
            onclick: move |_| {
                let mut users: Vec<NodeIdentity> = vec![];
                users.push(owner_id.read().clone());
                for x in ready_users.read().iter() {
                    users.push(x.clone());
                }
                users.truncate(2);
                let chat3_send = chat3_send.clone();
                let mm = mm.clone();

                async move {
                    let _match = GameMatch { 
                        match_id: uuid::Uuid::new_v4(), 
                        seed: get_random_seed(),
                         time: get_timestamp_now_ms(),
                          users,
                           title: "Private 1v1 {room_id}".to_string(), 
                           type_: game::api::game_match::GameMatchType::_1v1,
                    };
                    let _ = chat3_send.broadcast_message(PrivateLobbyMessage::StartPlaying(_match.clone())).await;
                    let url = Route::Play1v1Page {
                        game_match: UrlParam(_match),
                    }.to_string();
                    mm.sleep(Duration::from_millis(50)).await;
                    navigator().replace(url);
                }
            },
            "Start Game!",
        }
        h3 {"Ready Users: {ready_users.read().len()}"}
        ul {
            for x in ready_users.read().iter() {
               li {
                    style:"color:{x.html_color()}",
                    "{x.nickname()}"
               } 
            }
        }

    }
}

#[component]
fn RoomControlPlayer(
    our_id: ReadOnlySignal<NodeIdentity>,
    owner_id: ReadOnlySignal<NodeIdentity>,
    room_uuid: ReadOnlySignal<uuid::Uuid>,
    chat:  ChatSignals<PrivateLobyRoomType> ,
) -> Element {
    let mut clicked_ready = use_signal(|| false);
    let chat = chat.chat.read().clone();
    let Some(chat) = chat else {return rsx!{"no chat"}};
    let chat_send = chat.sender();

    if our_id.read().user_id().clone() == owner_id.read().user_id().clone() {
        return rsx! {
            h1 {"Player (also you)"}
            "You have opened your own link. It works. Please share it with another user."
        }
    }

    let chat2 = chat.clone();
    let _r = use_resource(move || {
        let chat2 = chat2.clone();
        async move {
            let rx = chat2.receiver().await;
            while let Some(msg) = rx.next_message().await {
                let _from = msg.from;
                let msg_content = &msg.message;
                let PrivateLobbyMessage::StartPlaying(_match) = msg_content else {
                    continue;
                };
                let _match = _match.clone();
                let url = Route::Play1v1Page {
                        game_match: UrlParam(_match),
                    }.to_string();
                navigator().replace(url);
            }
        }
    });

    rsx! {
        h1 {"Player"}
        button {
            onclick: move |_| {
                let chat_send = chat_send.clone();
                let chat = chat.clone();
                async move {
                    clicked_ready.set(true);
                    let _ = chat.wait_joined().await;
                    let _  = chat_send.broadcast_message(PrivateLobbyMessage::ReadyToPlay(true)).await;
            }},
            disabled: *clicked_ready.read(),
            "Ready to Play"
        }
    }
}

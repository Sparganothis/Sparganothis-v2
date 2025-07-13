#![allow(deprecated)]

use dioxus::prelude::*;
use protocol::user_identity::NodeIdentity;
use tracing::info;
use wasm_bindgen::prelude::*;
use crate::{comp::chat::private_lobby_chat::PrivateLobbyChatBox, network::NetworkState, route::{Route, UrlParam}};


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
pub fn PrivateLobbyPage(owner_id: ReadOnlySignal<UrlParam<NodeIdentity>>, room_uuid: ReadOnlySignal<uuid::Uuid>) -> Element {

    let url = use_memo(move || {
        let owner_id  = owner_id.read().clone();
        let room_uuid = room_uuid.read().clone();
        let _u1 = Route::PrivateLobbyPage {
            room_uuid,owner_id,
        }.to_string();
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
fn RoomControlComponent( owner_id:  ReadOnlySignal<NodeIdentity>, room_uuid: ReadOnlySignal<uuid::Uuid>) -> Element {


            let mm = use_context::<NetworkState>().global_mm;
    let Some(mm) = mm.read().clone() else {
        return rsx!{
            "loading"
        };
    };
    let our_id = mm.own_node_identity();

    rsx! {

        article {
            style:"padding:10px;margin:10px;",

            if our_id == owner_id.read().clone() {
                RoomControlAdmin{our_id, owner_id, room_uuid}
            } else {
                RoomControlPlayer{our_id, owner_id, room_uuid}
            }
        }
    }
}

#[component]
fn RoomControlAdmin(our_id: ReadOnlySignal<NodeIdentity>, owner_id:  ReadOnlySignal<NodeIdentity>, room_uuid: ReadOnlySignal<uuid::Uuid>) -> Element
{rsx!{
    h1 {"Admin"}
                button {
                disabled: true,
                "Start Game!",
            }

}}

#[component]
fn RoomControlPlayer(our_id: ReadOnlySignal<NodeIdentity>, owner_id:  ReadOnlySignal<NodeIdentity>, room_uuid: ReadOnlySignal<uuid::Uuid>) -> Element
{rsx!{
    h1 {"Player"}

                button {
                "Spectate"
            }
            button {
                "Ready to Play"
            }
}}
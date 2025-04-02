use crate::{
    comp::chat::{
        chat_window_fullscreen::FullscreenChatRoom,
        chat_window_mini::MiniChatRoomOverlay,
    },
    network::GlobalChatClientContext,
    route::Route,
};
use dioxus::prelude::*;

#[component]
pub fn GlobalMiniChatOverlayParent(children: Element) -> Element {
    let chat = use_context::<GlobalChatClientContext>().chat;
    let route = use_route::<Route>();
    let fullscreen = route == Route::GlobalChatPage {};
    rsx! {
        if !fullscreen {
            div {
                id: "GlobalMiniChatOverlayParent",
                style: "
                    width: 100%;
                    height: 100%;
                ",
                {children}
            }
            MiniChatRoomOverlay { chat }
        } else {
            FullscreenChatRoom { chat }
        }
    }
}

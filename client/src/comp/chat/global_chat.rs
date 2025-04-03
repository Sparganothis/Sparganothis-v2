use crate::{
    app::GlobalUrlContext,
    comp::chat::{
        chat_window_fullscreen::FullscreenChatRoom,
        chat_window_mini::MiniChatRoomOverlay,
    },
    network::GlobalChatClientContext,
    route::Route,
};
use dioxus::prelude::*;
use tracing::info;

#[component]
pub fn GlobalMiniChatOverlayParent(children: Element) -> Element {
    info!("GlobalMiniChatOverlayParent");

    rsx! {
        OverlayInner { children }
    }
}

#[component]
fn OverlayInner(children: Element) -> Element {
    let route = use_context::<GlobalUrlContext>().route;
    let fullscreen =
        use_memo(move || *route.read() == Route::GlobalChatPage {});
    let chat = use_context::<GlobalChatClientContext>().chat;
    rsx! {
        if !*fullscreen.read() {
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

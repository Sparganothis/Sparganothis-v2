use dioxus::prelude::*;

use crate::comp::multiplayer::_1v1::Play1v1FullscreenWindow;

#[component]
pub fn Play1v1Page() -> Element {
    rsx! {
        article {
            style: "
                height: 100%;
                width: 100%;
            ",

            Play1v1FullscreenWindow {
                
            }
        }
    }
}

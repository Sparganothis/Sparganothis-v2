use dioxus::prelude::*;

use crate::comp::singleplayer::SingleplayerGameBoardBasic;

/// Home page
#[component]
pub fn PlaySingleplayerPage() -> Element {
    rsx! {
        article {
            style: "height: 80dvh; display: flex;",
            SingleplayerGameBoardBasic {}
        }
    }
}

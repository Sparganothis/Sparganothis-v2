use dioxus::prelude::*;

use crate::comp::singleplayer::SingleplayerGameBoard;

/// Home page
#[component]
pub fn PlaySingleplayerPage() -> Element {
    rsx! {
        article {
            style: "height: 80dvh; display: flex;",
            SingleplayerGameBoard {}
        }
    }
}

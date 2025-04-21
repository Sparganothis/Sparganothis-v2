use dioxus::prelude::*;
use game::api::game_match::GameMatch;
use protocol::user_identity::NodeIdentity;

#[component]
pub fn Play1v1FullscreenWindow(game_match: GameMatch<NodeIdentity>) -> Element {
    rsx! {
        div {
            style: "width: 100%; height: 100%; flex-grow: 1;",
            "TODO play 1v11"
        }
    }
}

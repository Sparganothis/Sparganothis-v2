mod _1v1;
pub use _1v1::*;
mod matchmaking;
pub use matchmaking::*;

mod private_1v1_lobby;
pub use private_1v1_lobby::*;

use dioxus::prelude::*;

#[component]
pub fn PlayGameRootPage() -> Element {
    rsx! {
        "TODO page: PlayGameRootPage "
    }
}

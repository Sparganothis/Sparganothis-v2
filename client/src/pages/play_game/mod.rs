mod _1v1;
pub use _1v1::*;
mod matchmaking;
pub use matchmaking::*;

mod private_lobby;
pub use private_lobby::*;

mod _1v1_outcome;
pub use _1v1_outcome::*;

use dioxus::prelude::*;

#[component]
pub fn PlayGameRootPage() -> Element {
    rsx! {
        "TODO page: PlayGameRootPage "
    }
}

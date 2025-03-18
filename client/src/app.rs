use crate::{
    constants::APP_TITLE, localstorage::LocalStorageParent,
    network::NetworkConnectionParent, route::Route,
};
use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const PICO_CSS: Asset = asset!("/assets/pico.jade.min.css");
// const MAIN_CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: PICO_CSS }
        document::Link { rel: "icon", href: FAVICON }
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Title { "{APP_TITLE}" }
        div {
            "data-theme": "light",
            style: "
              background-color: var(--pico-background-color);
              color: var(--pico-color);
              width:100dvw;
              height:100dvh;
              margin:0;
              padding:0; 
              overflow: hidden;
            ",
            LocalStorageParent {
                NetworkConnectionParent {
                    Router::<Route> {}
                }
            }
        }
    }
}

use crate::{
     constants::APP_TITLE, localstorage::LocalStorageParent, network::NetworkConnectionParent, route::Route
};
use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const PICO_CSS: Asset = asset!("/assets/pico.jade.min.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: PICO_CSS }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Title { "{APP_TITLE}" }
        div {
            "data-theme": "light",
            class: "global_parent",
            UrlHolderParent {
                LocalStorageParent {
                    NetworkConnectionParent {
                        Router::<Route> {}
                    }
                }
            }
        }
    }
}

#[component]
fn UrlHolderParent(children: Element) -> Element {
    let url = use_signal(move || "".to_string());
    let url_r = use_memo(move || url.read().clone());
    use_context_provider(move || GlobalUrlContext {
        url_w: url.into(),
        url: url_r.into(),
    });
    children
}

#[derive(Clone, Debug)]
pub struct GlobalUrlContext {
    pub url_w: Signal<String>,
    pub url: ReadOnlySignal<String>,
}
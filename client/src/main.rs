use client::{constants::APP_TITLE, route::Route};
use dioxus::prelude::*;



const FAVICON: Asset = asset!("/assets/favicon.ico");
const PICO_CSS: Asset = asset!("/assets/pico.jade.min.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}




#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: PICO_CSS }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Title { "{APP_TITLE}" }
        Router::<Route> {}
    }
}


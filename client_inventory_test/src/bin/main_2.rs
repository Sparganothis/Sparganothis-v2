use dioxus::{
    logger::tracing::{info, warn},
    prelude::*,
};
// use protocol::{
    // global_matchmaker::GlobalMatchmaker,
    //   server_chat_api::client_api_manager::connect_api_manager, inventory,
    //  user_identity::UserIdentitySecrets
// };

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {}
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}


async fn inventory_demo() -> String {
    let mut v = vec![];
    for x in inventory::iter::<Rahan> {
        v.push(x.text.to_string())
    }
    v.join(", ")
}

pub struct Rahan {
    pub text: &'static str,
}
impl Rahan {
    pub const fn new(text: &'static str) -> Rahan {
        Rahan {
            text
        }
    }
}

use inventory;
inventory::collect!(Rahan);
inventory::submit! { Rahan::new("Rahan") }
inventory::submit! { Rahan::new("Rahan2") }
inventory::submit! { Rahan::new("Rahan23") }



#[component]
pub fn Hero() -> Element {
    let mut msg = use_signal(String::new);
    let _coro =
        use_coroutine(move |mut _m_b: UnboundedReceiver<()>| async move {
            let s = inventory_demo().await;
            info!("INVENTORY DEMO = {}", &s);
            msg.set(s);

            warn!("XXX: Network connection parent coroutine exited");
        });
    rsx! {
        pre {
            "{msg}"
        }
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        Hero {}

    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
        }

        Outlet::<Route> {}
    }
}

use std::sync::Arc;

use dioxus::{
    html::{script::r#async, u::user_select},
    logger::tracing::warn,
    prelude::*,
};
use game::futures_util::StreamExt;
use protocol::{
    global_matchmaker::GlobalMatchmaker,
    server_chat_api::client_api_manager::connect_api_manager,
    user_identity::UserIdentitySecrets,
};

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

#[component]
pub fn Hero() -> Element {
    let mut mm = use_signal(|| None);
    let mut msg = use_signal(String::new);
    let _coro =
        use_coroutine(move |mut m_b: UnboundedReceiver<()>| async move {
            let user_secrets = UserIdentitySecrets::generate();
            let user_secrets = Arc::new(user_secrets);
            match GlobalMatchmaker::new(user_secrets).await {
                Ok(client) => {
                    let api = connect_api_manager(client.clone()).await;
                    match api {
                        Ok(api) => {
                            mm.set(Some((client, api)));
                        }
                        Err(_) => warn!("asdfslkajhfalkjfdalkja"),
                    }
                }
                Err(e) => {
                    warn!("Failed to connect to global matchmaker: {e}");
                }
            };

            warn!("XXX: Network connection parent coroutine exited");
        });
    use_resource(move || {
        let mm = mm.read().clone();
        async move {
            let Some((mm, api)) = mm else {
                return;
            };
        }
    });
    rsx! {
    pre {
        "{mm.read().clone():#?}"
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

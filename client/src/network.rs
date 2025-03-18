use std::sync::Arc;

use dioxus::prelude::*;
use iroh::SecretKey;
use n0_future::StreamExt;
use protocol::{
    chat::NetworkEvent, global_matchmaker::GlobalMatchmaker, user_identity::UserIdentitySecrets,
};
use tracing::warn;

use crate::{comp::modal::ModalArticle, localstorage::LocalStorageContext};

#[derive(Clone)]
pub struct NetworkState {
    pub global_mm: ReadOnlySignal<Option<GlobalMatchmaker>>,
    pub global_mm_loading: ReadOnlySignal<bool>,
    pub is_connected: ReadOnlySignal<bool>,
    pub reset_network: Callback<()>,
}

#[component]
pub fn NetworkConnectionParent(children: Element) -> Element {
    let mut mm_signal = use_signal(move || None);
    let mut mm_signal_loading = use_signal(move || false);
    let mut is_connected = use_signal(move || false);
    let mm_signal_ = mm_signal.clone();

    let _coro = use_coroutine(move |mut m_b: UnboundedReceiver<()>| async move {
        mm_signal_loading.set(true);
        is_connected.set(false);
        let user_secrets = use_context::<LocalStorageContext>()
            .user_secrets
            .peek()
            .clone();
        let mut c = match client_connect(user_secrets.clone()).await {
            Ok(client) => {
                mm_signal.set(Some(client.clone()));
                is_connected.set(true);
                Some(client)
            }
            Err(e) => {
                warn!("Failed to connect to global matchmaker: {e}");
                None
            }
        };
        mm_signal_loading.set(false);

        while let Some(_x) = m_b.next().await {
            mm_signal.set(None);
            mm_signal_loading.set(true);
            is_connected.set(false);
            if let Some(client) = c.take() {
                if let Err(e) = client.shutdown().await {
                    warn!("Failed to shutdown global matchmaker: {e}");
                }
                drop(client);
            }
            c = match client_connect(user_secrets.clone()).await {
                Ok(client) => {
                    mm_signal.set(Some(client.clone()));
                    is_connected.set(true);
                    Some(client)
                }
                Err(e) => {
                    warn!("Failed to connect to global matchmaker: {e}");
                    None
                }
            };
            mm_signal_loading.set(false);
        }
    });
    let reset_network = Callback::new(move |_: ()| {
        _coro.send(());
    });
    use_context_provider(move || NetworkState {
        global_mm: mm_signal_.into(),
        global_mm_loading: mm_signal_loading.into(),
        reset_network: reset_network.clone(),
        is_connected: is_connected.into(),
    });

    rsx! {
        {children}
    }
}

pub async fn client_connect(
    user_secrets: Arc<UserIdentitySecrets>,
) -> anyhow::Result<GlobalMatchmaker> {
    let global_mm = GlobalMatchmaker::new(user_secrets).await?;
    Ok(global_mm)
}

#[component]
pub fn NetworkConnectionStatusIcon() -> Element {
    let NetworkState {
        global_mm,
        reset_network,
        global_mm_loading,
        ..
    } = use_context::<NetworkState>();

    let net_txt = use_memo(move || {
        if global_mm.read().is_some() {
            "ONLINE"
        } else {
            "OFFLINE"
        }
    });
    let net_txt_color = use_memo(move || {
        if global_mm.read().is_some() {
            "blue"
        } else {
            "red"
        }
    });
    let mut modal_open = use_signal(move || false);
    let on_close = Callback::new(move |_: ()| {
        modal_open.set(false);
    });
    rsx! {
        article {
            style: "
            padding: 0px;
            margin: 0px;
            cursor: pointer;
            display: flex;
            ",
            onclick: move |_| {
                let t = !*modal_open.peek();
                modal_open.set(t);
            },
            h3 {
                "aria-busy": "{global_mm_loading.read()}",
                style: "margin: 0; color: {net_txt_color};",
                "{net_txt}",
            }
        }
        if *modal_open.read() {
            ModalArticle {
                on_close: on_close,
                title: rsx! {
                    h1 { "Network Connection" }
                    button {
                        onclick: move |_| reset_network.call(()),
                        "Reset Network"
                    }
                 },
                body: rsx! { NetworkConnectionDebugInfo {} },
            }
        }
    }
}

#[component]
fn NetworkConnectionDebugInfo() -> Element {
    let net = use_context::<NetworkState>();
    let mm_info = use_resource(move || {
        let mm = net.global_mm.read().clone();
        async move {
            let Some(mm) = mm else {
                return "No network connection".to_string();
            };
            mm.display_debug_info()
                .await
                .unwrap_or_else(|e| e.to_string())
        }
    });
    rsx! {
        if let Some(info) = mm_info.read().clone() {
            small {
                pre {
                    "{info}"
                }
            }
        }
    }
}

use std::sync::Arc;

use dioxus::prelude::*;
use n0_future::StreamExt;
use protocol::{
    _const::PRESENCE_INTERVAL,
    chat::{IChatController, IChatSender},
    global_matchmaker::GlobalMatchmaker,
    global_chat::{
        GlobalChatMessageType, GlobalChatPresence,},
    user_identity::UserIdentitySecrets,
};
use tracing::{info, warn};

use crate::{
    app::GlobalUrlContext,
    comp::{
        chat::chat_signals_hook::{
            use_global_chat_controller_signal, ChatSignals,
        },
        modal::ModalArticle,
    },
    localstorage::LocalStorageContext,
};

#[derive(Clone)]
pub struct NetworkState {
    pub global_mm: ReadOnlySignal<Option<GlobalMatchmaker>>,
    pub global_mm_loading: ReadOnlySignal<bool>,
    pub is_connected: ReadOnlySignal<bool>,
    pub reset_network: Callback<()>,
    pub bootstrap_idx: ReadOnlySignal<Option<u32>>,
    pub debug_info_txt: ReadOnlySignal<String>,
}

#[component]
pub fn NetworkConnectionParent(children: Element) -> Element {
    info!("NetworkConnectionParent");
    rsx! {
        GlobalMatchmakerParent {
            GlobalChatClientParent {
                {children}
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalChatClientContext {
    pub chat: ChatSignals<GlobalChatMessageType>,
}

#[component]
fn GlobalChatClientParent(children: Element) -> Element {
    info!("GlobalChatClientParent");
    let chat = use_global_chat_controller_signal();
    use_context_provider(move || GlobalChatClientContext { chat });
    rsx! {
        {children}
    }
}

#[component]
fn GlobalMatchmakerParent(children: Element) -> Element {
    info!("GlobalMatchmakerParent");
    let url = use_context::<GlobalUrlContext>().url;
    let mut mm_signal_w = use_signal(move || None);
    let mm_signal = use_memo(move || mm_signal_w.read().clone());

    let mut mm_signal_loading_w = use_signal(move || false);
    let mm_signal_loading =
        use_memo(move || mm_signal_loading_w.read().clone());

    let mut is_connected_w = use_signal(move || false);
    let is_connected = use_memo(move || is_connected_w.read().clone());

    let mut debug_info_txt_w = use_signal(move || "".to_string());
    let debug_info_txt = use_memo(move || debug_info_txt_w.read().clone());

    let mut bootstrap_idx_w: Signal<Option<u32>> = use_signal(move || None);
    let bootstrap_idx = use_memo(move || bootstrap_idx_w.read().clone());

    let _coro =
        use_coroutine(move |mut m_b: UnboundedReceiver<()>| async move {
            mm_signal_loading_w.set(true);
            is_connected_w.set(false);
            let user_secrets = use_context::<LocalStorageContext>()
                .persistent
                .user_secrets
                .peek()
                .clone();
            let mut c = match client_connect(user_secrets.clone()).await {
                Ok(client) => {
                    mm_signal_w.set(Some(client.clone()));
                    is_connected_w.set(true);
                    Some(client)
                }
                Err(e) => {
                    warn!("Failed to connect to global matchmaker: {e}");
                    None
                }
            };
            mm_signal_loading_w.set(false);

            while let Some(_x) = m_b.next().await {
                mm_signal_w.set(None);
                mm_signal_loading_w.set(true);
                is_connected_w.set(false);
                if let Some(client) = c.take() {
                    if let Err(e) = client.shutdown().await {
                        warn!("Failed to shutdown global matchmaker: {e}");
                    }
                    drop(client);
                }
                c = match client_connect(user_secrets.clone()).await {
                    Ok(client) => {
                        mm_signal_w.set(Some(client.clone()));
                        is_connected_w.set(true);
                        Some(client)
                    }
                    Err(e) => {
                        warn!("Failed to connect to global matchmaker: {e}");
                        None
                    }
                };
                mm_signal_loading_w.set(false);
            }
            warn!("XXX: Network connection parent coroutine exited");
        });
    let reset_network = Callback::new(move |_: ()| {
        _coro.send(());
    });
    let _poll_presence = use_resource(move || {
        let mm = mm_signal.read().clone();
        async move {
            let Some(mm) = mm else {
                debug_info_txt_w.set("No network connection".to_string());
                return;
            };
            let Some(presence) =
                mm.global_chat_controller().await.map(|c| c.chat_presence())
            else {
                debug_info_txt_w.set("No chat controller".to_string());
                return;
            };
            loop {
                debug_info_txt_w.set(
                    mm.display_debug_info()
                        .await
                        .unwrap_or_else(|e| e.to_string()),
                );
                n0_future::future::race(
                    mm.sleep(PRESENCE_INTERVAL),
                    presence.notified(),
                )
                .await;
                bootstrap_idx_w.set(
                    mm.bs_node()
                        .await
                        .map(|n| n.node_identity().bootstrap_idx())
                        .flatten(),
                );
            }
        }
    });
    // restarting resource that updates our global presence
    let _ = use_resource(move || {
        let url = url.read().clone();
        let platform = "browser".to_string();
        let presence = GlobalChatPresence { url, platform };
        let mm = mm_signal.read().clone();
        async move {
            let Some(mm) = mm else {
                return;
            };
            let Some(cc) = mm.global_chat_controller().await else {
                return;
            };
            cc.sender().set_presence(&presence).await;
        }
    });

    use_context_provider(move || NetworkState {
        global_mm: mm_signal.into(),
        global_mm_loading: mm_signal_loading.into(),
        reset_network: reset_network.clone(),
        is_connected: is_connected.into(),
        debug_info_txt: debug_info_txt.into(),
        bootstrap_idx: bootstrap_idx.into(),
    });

    children
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
        bootstrap_idx,
        ..
    } = use_context::<NetworkState>();

    let net_txt = use_memo(move || {
        let bs_idx = *bootstrap_idx.read();
        if global_mm.read().is_some() {
            if let Some(bs_idx) = bs_idx {
                format!("ONLINE #{}", bs_idx)
            } else {
                "ONLINE".to_string()
            }
        } else {
            "OFFLINE".to_string()
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
    let info = use_context::<NetworkState>().debug_info_txt;
    rsx! {
        small {
            pre {
                "{info}"
            }
        }
    }
}

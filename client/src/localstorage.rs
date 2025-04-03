use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_sdk::storage::{
    use_storage, use_synced_storage, LocalStorage, SessionStorage,
};
use protocol::user_identity::UserIdentitySecrets;
use tracing::info;

use crate::comp::chat::chat_window_mini::MiniChatTabSelection;

#[derive(Clone)]
pub struct LocalStorageContext {
    pub persistent: LocalPersistentStorage,
    pub session: LocalSessionStorage,
}

#[derive(Clone)]
pub struct LocalPersistentStorage {
    pub user_secrets: ReadOnlySignal<Arc<UserIdentitySecrets>>,
}

#[derive(Clone)]
pub struct LocalSessionStorage {
    pub tab_select: Signal<MiniChatTabSelection>,
}

#[component]
pub fn LocalStorageParent(children: Element) -> Element {
    info!("LocalStorageParent");
    let user_secrets =
        use_synced_storage::<LocalStorage, Arc<UserIdentitySecrets>>(
            "user_secrets_3".to_string(),
            || Arc::new(UserIdentitySecrets::generate()),
        );
    let user_secrets: ReadOnlySignal<Arc<UserIdentitySecrets>> =
        use_memo(move || user_secrets.read().clone()).into();
    use_effect(move || {
        info!("REFRESH user_secrets: {:#?}", user_secrets.read());
    });
    let tab_select = use_storage::<SessionStorage, MiniChatTabSelection>(
        "tab_select_signal".to_string(),
        || MiniChatTabSelection::Minified,
    );
    use_context_provider(move || LocalStorageContext {
        persistent: LocalPersistentStorage { user_secrets },
        session: LocalSessionStorage { tab_select },
    });

    children
}

use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_sdk::storage::{use_synced_storage, LocalStorage};
use protocol::user_identity::UserIdentitySecrets;
use tracing::info;

#[derive(Clone)]
pub struct LocalStorageContext {
    pub user_secrets: ReadOnlySignal<Arc<UserIdentitySecrets>>,
}

#[component]
pub fn LocalStorageParent(children: Element) -> Element {
    let user_secrets =
        use_synced_storage::<LocalStorage, Arc<UserIdentitySecrets>>(
            "user_secrets_3".to_string(),
            || Arc::new(UserIdentitySecrets::generate()),
        );
    let user_secrets: ReadOnlySignal<Arc<UserIdentitySecrets>> = use_memo(move || user_secrets.read().clone()).into();
    use_effect(move || {
        info!("REFRESH user_secrets: {:#?}", user_secrets.read());
    });
    use_context_provider(move || LocalStorageContext { user_secrets });

    children
}

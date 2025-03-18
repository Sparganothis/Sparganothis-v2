use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_sdk::storage::{use_synced_storage, LocalStorage};
use protocol::user_identity::UserIdentitySecrets;

#[derive(Clone)]
pub struct LocalStorageContext {
    pub user_secrets: Signal<Arc<UserIdentitySecrets>>,
}

#[component]
pub fn LocalStorageParent(children: Element) -> Element {
    let user_secrets = use_synced_storage::<LocalStorage, Arc<UserIdentitySecrets>>(
        "user_secrets2".to_string(),
        || Arc::new(UserIdentitySecrets::generate()),
    );
    use_context_provider(move || LocalStorageContext { user_secrets });

    children
}

use dioxus::prelude::*;
use dioxus_sdk::storage::{use_synced_storage, LocalStorage};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct LocalStorageContext {
    pub user_id: Signal<Uuid>,
}

#[component]
pub fn LocalStorageParent(children: Element) -> Element {
    let user_id =
        use_synced_storage::<LocalStorage, Uuid>("user_id".to_string(), || Uuid::new_v4());
    use_context_provider(move || LocalStorageContext { user_id });

    children
}

use dioxus::prelude::*;
use dioxus_sdk::storage::*;

#[component]
pub fn StorageDemo() -> Element {
    let mut count_session = use_singleton_persistent(|| 0);
    let mut count_local =
        use_synced_storage::<LocalStorage, i32>("synced".to_string(), || 0);

    rsx!(
        div {
            button {
                onclick: move |_| {
                    *count_session.write() += 1;
                },
                "Click me!"
            },
            "I persist for the current session. Clicked {count_session} times"
        }
        div {
            button {
                onclick: move |_| {
                    *count_local.write() += 1;
                },
                "Click me!"
            },
            "I persist across all sessions. Clicked {count_local} times"
        }
    )
}

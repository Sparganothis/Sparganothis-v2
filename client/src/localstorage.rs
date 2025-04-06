use std::sync::Arc;

use dioxus::prelude::*;
use dioxus_sdk::storage::{
    use_storage, use_synced_storage, LocalStorage, SessionStorage,
};
use game::settings::GameSettings;
use protocol::user_identity::UserIdentitySecrets;
use tracing::info;

use crate::comp::{chat::chat_window_mini::MiniChatTabSelection, controls_button_form::ButtonSettings};

#[derive(Clone)]
pub struct LocalStorageContext {
    pub persistent: LocalPersistentStorage,
    pub session: LocalSessionStorage,
}

#[derive(Clone)]
pub struct LocalPersistentStorage {
    pub user_secrets: ReadOnlySignal<Arc<UserIdentitySecrets>>,
    pub game_settings: ReadOnlySignal<GameSettings>,
    __game_settings_w: Signal<GameSettings>,

    pub button_settings: ReadOnlySignal<ButtonSettings>,
    __button_settings_w: Signal<ButtonSettings>,
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
    let game_settings_w = 
        use_synced_storage::<LocalStorage, GameSettings>(
            "game_settings_1".to_string(),
            || GameSettings::default(),
        );
    let game_settings = use_memo(move || game_settings_w.read().clone());
    
    let button_settings_w = use_synced_storage::<LocalStorage, ButtonSettings>(
        "button_settings_1".to_string(),
        || ButtonSettings::default(),
    );
    let button_settings = use_memo(move || button_settings_w.read().clone());

    use_context_provider(move || LocalStorageContext {
        persistent: LocalPersistentStorage { 
            user_secrets,
            game_settings: game_settings.into(),
            __game_settings_w: game_settings_w,
            button_settings: button_settings.into(),
            __button_settings_w: button_settings_w,
        },
        session: LocalSessionStorage { tab_select },
    });

    children
}

pub fn use_game_settings() -> GameSettings {
    let x = use_context::<LocalStorageContext>().persistent.game_settings.read().clone();
    x
}

pub fn set_game_settings(s: GameSettings)  {
    let mut z = use_context::<LocalStorageContext>().persistent.__game_settings_w;
    z.set(s);
}

pub fn use_button_settings() -> ButtonSettings {
    let x = use_context::<LocalStorageContext>().persistent.button_settings.read().clone();
    x
}

pub fn set_button_settings(s: ButtonSettings)  {
    let mut z = use_context::<LocalStorageContext>().persistent.__button_settings_w;
    z.set(s);
}
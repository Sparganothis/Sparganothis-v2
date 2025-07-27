use dioxus::prelude::*;
use protocol::{
    api::{
        api_declarations::GetUserProfile, client_api_manager::ClientApiManager,
    },
    global_matchmaker::GlobalMatchmaker,
    user_identity::UserIdentity,
};

use crate::comp::users::top_players_tables::DisplayUserProfileCard;

#[component]
pub fn UserProfileDisplay(
    api: ReadOnlySignal<ClientApiManager>,
    mm: ReadOnlySignal<GlobalMatchmaker>,
    user_id: ReadOnlySignal<UserIdentity>,
) -> Element {
    let nickname = user_id.read().nickname();
    let mut err = use_signal(String::new);

    let data = use_resource(move || {
        let api = api.read().clone();
        let user_id = user_id.read().clone();
        async move {
            let x = api
                .call_method::<GetUserProfile>(user_id)
                .await
                .map_err(|e| format!("{e:#?}"));
            if let Err(ref e) = x {
                err.set(e.to_string());
            };
            x
        }
    });
    let data = use_memo(move || data.read().clone().map(|x| x.ok()).flatten());

    rsx! {

        h1 {
            "User \"{nickname}\""
        }
        if let Some(d) = data.read().as_ref() {
            DisplayUserProfileCard {item: d.clone()}
        }
        if err.read().len() > 0 {
            div {
                style:"color:red;",
                "{err}"
            }
        }
    }
}

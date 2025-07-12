use dioxus::prelude::*;
use protocol::user_identity::UserIdentity;

use crate::{
    comp::users::{
        top_players_tables::PlayersWithMostMatchesTable,
        user_profile_display::UserProfileDisplay,
    },
    network::NetworkState,
    route::UrlParam,
};

#[component]
pub fn UsersRootDirectoryPage() -> Element {
    let net = use_context::<NetworkState>();
    let mm = net.global_mm;
    let api = net.client_api_manager;
    let (Some(mm), Some(api)) = (mm.read().clone(), api.read().clone()) else {
        return rsx! {
            "loading..."
        };
    };

    rsx! {
        div {
            class: "container",
            h1 {
                "Players with most matches"
            }
            PlayersWithMostMatchesTable{api, mm}
        }
    }
}

#[component]
pub fn UsersProfilePage(
    user_id: ReadOnlySignal<UrlParam<UserIdentity>>,
) -> Element {
    let user_id = use_memo(move || user_id.read().0.clone());
    let net = use_context::<NetworkState>();
    let mm = net.global_mm;
    let api = net.client_api_manager;
    let (Some(mm), Some(api)) = (mm.read().clone(), api.read().clone()) else {
        return rsx! {
            "loading..."
        };
    };
    let user_id = user_id.read().clone();

    rsx! {
        article {
            style: "container",

            UserProfileDisplay {
                api, mm, user_id
            }
        }
    }
}

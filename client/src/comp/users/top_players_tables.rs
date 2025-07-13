use dioxus::prelude::*;
use game::timestamp::get_timestamp_now_ms;
use protocol::{
    global_matchmaker::GlobalMatchmaker,
    server_chat_api::{
        api_declarations::{GetUsersWithTopGameCounts, UserProfileListItem},
        client_api_manager::ClientApiManager,
    },
};

use crate::route::{Route, UrlParam};

#[component]
pub fn PlayersWithMostMatchesTable(
    api: ReadOnlySignal<ClientApiManager>,
    mm: ReadOnlySignal<GlobalMatchmaker>,
) -> Element {
    let data = use_resource(move || {
        let api = api.read().clone();
        async move {
            api.call_method::<GetUsersWithTopGameCounts>(())
                .await
                .map_err(|e| format!("{e:?}"))
        }
    });
    let data = use_memo(move || data.read().clone().map(|x| x.ok()).flatten());

    let Some(data) = data.read().clone() else {
        return rsx! {"loading..."};
    };

    rsx! {
        article {
            class: "container",
            style: "display: flex; flex-direction: column; width:688px;",

            for item in data {
                Link {
                    to: Route::UsersProfilePage { user_id: UrlParam(item.user.clone()) },
                    DisplayUserProfileCard {item}
                }
            }
        }
    }
}

#[component]
pub fn DisplayUserProfileCard(item: UserProfileListItem) -> Element {
    let nickname = item.user.nickname();
    let color = item.user.html_color();
    let now = get_timestamp_now_ms();

    let days_since_create = (now - item.first_login) / 86400 / 1000;
    let days_since_login = (now - item.last_login) / 86400 / 1000;
    let game_count = item.game_count;

    rsx! {
        article {
            style: "width: 666px; border: 1px solid {color}; color: {color}; overflow:hidden;",

            h3 {     "       {nickname}    --   {game_count} games   " }
            p {
                "Days since created account: {days_since_create}. Days since last login: {days_since_login}."
            }
        }
    }
}

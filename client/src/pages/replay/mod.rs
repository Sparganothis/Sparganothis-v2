use dioxus::{html::elements, prelude::*};
use protocol::server_chat_api::api_declarations::{
    GetGameStateRowsForMatch, GetReplayMatchDetail, GetReplayMatchList,
    MatchRow2,
};

use crate::{network::NetworkState, route::Route};

#[component]
pub fn ReplayHomePage() -> Element {
    let err = use_signal(move || String::new());
    let mut data = use_signal(move || vec![]);

    let api = use_context::<NetworkState>().client_api_manager;
    let api = api.read().clone();
    let Some(api) = api else {
        return rsx! {"loading..."};
    };

    let api2 = api.clone();
    let _r = use_resource(move || {
        let api2 = api2.clone();

        async move {
            let Ok(r) = api2.call_method::<GetReplayMatchList>(()).await else {
                tracing::warn!("call error!");
                return ();
            };
            data.set(r);
            return ();
        }
    });

    rsx! {
        if !err.read().is_empty()
        {
            {err}
        } else {
            h1 {
                "Replay Match List"
            }
            ul {
                for d in data.read().clone().iter() {
                    li {
                        Link {
                            to: Route::Replay1v1Match { match_id: d.match_id.clone() },
                            style:"max-height: 200px; margin:20px; padding:20px; border: 1px solid black; display:flex;",
                            pre {
                                "{d:#?}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Replay1v1Match(match_id: ReadOnlySignal<String>) -> Element {
    let err = use_signal(move || String::new());
    let mut match_row = use_signal(move || None);

    let api = use_context::<NetworkState>().client_api_manager;
    let api = api.read().clone();
    let Some(api) = api else {
        return rsx! {"loading..."};
    };

    let api2 = api.clone();
    let _r = use_resource(move || {
        let api2 = api2.clone();
        let match_id = match_id.read().clone();
        async move {
            let Ok(r) =
                api2.call_method::<GetReplayMatchDetail>(match_id).await
            else {
                tracing::warn!("call error!");
                return ();
            };
            match_row.set(Some(r));
            return ();
        }
    });

    rsx! {

        if !err.read().is_empty()
        {
            {err}
        } else {
            h1 {
                "Match ID: {match_id}"
            }
            if let Some(match_row) = match_row.read().as_ref() {
                DisplayReplayDetails {match_info: match_row.clone()}
            }
        }
    }
}

#[component]
fn DisplayReplayDetails(match_info: ReadOnlySignal<MatchRow2>) -> Element {
    let match_info = use_memo(move || match_info.read().clone());
    let match_info = match_info.read().clone();
    let err = use_signal(move || String::new());
    let mut match_row = use_signal(move || vec![]);

    let api = use_context::<NetworkState>().client_api_manager;
    let api = api.read().clone();
    let Some(api) = api else {
        return rsx! {"loading..."};
    };

    let api2 = api.clone();
    let _r = use_resource(move || {
        let api2 = api2.clone();
        let match_id = match_info.clone();
        async move {
            let Ok(r) =
                api2.call_method::<GetGameStateRowsForMatch>(match_id).await
            else {
                tracing::warn!("call error!");
                return ();
            };
            match_row.set(r);
            return ();
        }
    });

    rsx! {

        if !err.read().is_empty()
        {
            {err}
        } else {
            h1 {
                "STATE LIST!"
            }
            for x in match_row.read().iter() {
                li {
                    pre {
                        "{x:?}"
                    }
                }
            }
        }
    }
}

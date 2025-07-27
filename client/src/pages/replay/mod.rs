use std::collections::HashMap;

use dioxus::prelude::*;
use game::tet::GameState;
use protocol::api::api_declarations::{
    GetGameStateRowsForMatch, GetReplayMatchDetail, GetReplayMatchList,
    MatchRow2,
};

use crate::{
    comp::{game_display::GameDisplay, slider::Slider},
    network::NetworkState,
    route::Route,
};

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
    let m2 = match_info.clone();
    let _r = use_resource(move || {
        let api2 = api2.clone();
        let match_id = m2.clone();
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

    let rows2 = use_memo(move || {
        let rows1 = match_row.read().clone();
        let mut rows2 = HashMap::<String, Vec<GameState>>::new();
        for row in rows1 {
            let ins = rows2.entry(row.user_id).or_default();
            let Some(state) = &row.state_data else {
                continue;
            };
            ins.push(state.clone());
        }

        rows2
    });

    rsx! {

        if !err.read().is_empty()
        {
            {err}
        } else {
            div {
                style: "width: 100%;
                 height: 100%; flex-direction:row; display:flex;",
                for x in rows2.read().iter() {
                    GameStateBrowser {data: (x.1).clone()}
                }
            }
        }
    }
}

#[component]
pub fn GameStateBrowser(data: ReadOnlySignal<Vec<GameState>>) -> Element {
    let idx = use_signal(move || 0);
    let max = use_signal(move || {
        let i = data.read().len() as i32;
        tracing::warn!("\n \n\n MAXMAXMXAMX : {i} \n\n");
        i
    });

    let state = use_memo(move || {
        let idx = *idx.read();
        let vec = data.read();
        let max_idx = vec.len();
        if max_idx == 0 {
            return None;
        }
        let mut idx = idx as usize;
        if idx >= max_idx {
            idx = max_idx - 1;
        }
        vec.get(idx).cloned()
    });

    rsx! {
        div {
            style: "display:flex; width:40%;height:100%;flex-direction:column;",
            Slider {
                label: "SLIDER".to_string(),
                m: idx,
                default_value: 0,
                min: 0,
                max: *max.read()
            }
            "max: {*max.read()}"

            div {
                style: "width: 100%; height: 80%;border:1px solid black;",
                if let Some(state) = state.read().as_ref() {
                    GameDisplay {game_state: state.clone()}
                }
            }
        }
    }
}

use dioxus::{html::elements, prelude::*};
use protocol::server_chat_api::api_declarations::GetReplayMatchList;

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
        let Ok(r) = api2.call_method::<GetReplayMatchList>(()).await
         else {
            tracing::warn!("call error!");
            return ();
        };
        data.set(r);
        return ();
    }});

    rsx! {
        if !err.read().is_empty() 
        {
            {err}
        } else {
            ul {
                for d in data.read().clone().iter() {
                    li {
                        Link {
                            to: Route::Replay1v1Match { match_id: d.match_id.clone() },
                            style:"max-height: 100px; margin:20px; padding:20px; border: 1px solid black;",
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
pub fn Replay1v1Match(match_id: String) -> Element {
    rsx! {
        h1 {
            "Match ID: {match_id}"
        }
    }
}

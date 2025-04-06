use std::collections::BTreeMap;

use crate::localstorage::use_button_settings;

use dioxus::prelude::*;
use game::input::events::GameInputEventKey;

#[component]
pub fn ButtonsForm() -> Element {
    let btn_map = use_button_settings().map;
    let mut btn2 = BTreeMap::<GameInputEventKey, Vec<Code>>::new();

    for (btn_code, btn_event) in btn_map.into_iter() {
        let entry = btn2.entry(btn_event).or_insert(vec![]);
        entry.push(btn_code);
    }
    let keys = btn2.keys().cloned().collect::<Vec<_>>();
    let mut btn2 = btn2;
    for btn_event in keys {
        let entry = btn2.entry(btn_event).or_insert(vec![]);
        entry.sort_by_key(|c| {
            let b = bincode::serialize(&c).unwrap();
            b
        });
    }

    rsx! {
        article {
            style: "
                display: grid;  
                grid-template-columns: auto auto auto auto;
                padding: 1px;
                width: 100%;
                height: 100%;
            ",

            for (btn_event, btn_codes) in btn2 {
                ButtonSelector {
                    btn_event,
                    btn_codes
                }
            }
        }
    }
}

#[component]
fn ButtonSelector(
    btn_event: GameInputEventKey,
    btn_codes: Vec<Code>,
) -> Element {
    rsx! {
        div {
            style: "
                width: 100%;
                height: 100%;
                border: 1px solid pink;
                padding: 1px;
                margin: 1px;
            ",
            small {
                b {
                pre {
"{btn_event:#?}

{btn_codes:#?}",
                }
            }  }
        }
    }
}

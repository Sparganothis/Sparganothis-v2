
use crate::localstorage::use_button_settings;

use dioxus::prelude::*;
use game::input::events::GameInputEventKey;

#[component]
pub fn ButtonsForm() -> Element {
    let btn_map = use_button_settings().map;

    rsx! {
        article {
            style: "
                display: grid;  
                grid-template-columns: auto auto auto;
                padding: 1px;
                width: 100%;
                height: 100%;
            ",

            for (btn_code, btn_event) in btn_map.iter() {
                Button {
                    btn_code: *btn_code, 
                    btn_event: *btn_event,
                }
            }
        }
    }
}


#[component]
fn Button(btn_code: Code, btn_event: GameInputEventKey) -> Element {
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
"{btn_code:#?}

{btn_event:#?}",
                }
            }  }
        }   
    }
}
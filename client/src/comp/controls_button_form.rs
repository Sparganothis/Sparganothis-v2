
use dioxus::prelude::*;

use crate::comp::settings_form::GameSettingsInputPreview;
#[component]
pub fn 
GameControlsButtonsForm () -> Element {
    rsx! {       
        article {
        style: "
            display: flex;
            height: 55dvh;
            width: 100%;
            flex-direction: row;
        ",
        div {
            style: "
                height: 100%;
                width: 50%;
                border: 1px solid green;
                padding: 10px;
                margin: 10px;
            ",
            ButtonsForm {}
        }
        div {
            style: "
                height: 100%;
                width: 50%;
                border: 1px solid magenta;
                padding: 10px;
                margin: 10px;
            ",
            GameSettingsInputPreview {}
           
        }
    }
    }
}

#[component]
fn ButtonsForm() -> Element {
    rsx! {
        "Buttons Form"
    }
}
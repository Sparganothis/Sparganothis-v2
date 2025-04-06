use dioxus::prelude::*;

use crate::comp::controls_button_form::GameControlsButtonsForm;

#[component]
pub fn MyButtonSettings() -> Element {
    rsx! {
        GameControlsButtonsForm {  }
    }
}

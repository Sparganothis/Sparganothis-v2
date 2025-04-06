use dioxus::prelude::*;

use crate::comp::{controls_button_form::GameControlsButtonsForm, settings_form::SettingsForm};

#[component]
pub fn MyButtonSettings() -> Element {
    rsx! {
        GameControlsButtonsForm {  }
    }
}

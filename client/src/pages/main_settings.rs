use dioxus::prelude::*;

use crate::comp::settings_form::SettingsForm;

#[component]
pub fn MyMainSettings() -> Element {
    rsx! {
        article {
            style: "
                height: 100%;
            ",
            SettingsForm {  }
        }
    }
}

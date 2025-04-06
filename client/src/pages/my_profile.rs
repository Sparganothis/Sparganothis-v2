use dioxus::prelude::*;

use crate::comp::{
    cosmetic::Hline, settings_form::SettingsForm,
    user_info_display::CurrentUserInfoDisplay,
};

#[component]
pub fn MyProfilePage() -> Element {
    rsx! {
        article {
            CurrentUserInfoDisplay {}
        }
    }
}

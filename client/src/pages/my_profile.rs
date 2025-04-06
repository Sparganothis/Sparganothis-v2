use dioxus::prelude::*;

use crate::comp::{settings_form::SettingsForm, user_info_display::CurrentUserInfoDisplay, cosmetic::Hline};

#[component]
pub fn MyProfilePage() -> Element {
    rsx! {
        article {
            CurrentUserInfoDisplay {}
        }
    }
}

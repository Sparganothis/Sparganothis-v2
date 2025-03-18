use dioxus::prelude::*;

use crate::comp::user_info_display::CurrentUserInfoDisplay;

#[component]
pub fn MyProfilePage() -> Element {
    rsx! {
        CurrentUserInfoDisplay {}
    }
}

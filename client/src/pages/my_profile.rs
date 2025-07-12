use dioxus::prelude::*;

use crate::comp::users::current_user_info_display::CurrentUserInfoDisplay;

#[component]
pub fn MyProfilePage() -> Element {
    rsx! {
        article {
            CurrentUserInfoDisplay {}
        }
    }
}

use dioxus::prelude::*;
use dioxus_free_icons::IconShape;

#[component]
pub fn Icon<T: IconShape + Clone + PartialEq + 'static>(
    icon: T,
    color: String,
    selected: bool,
    onclick: Callback<()>,
    tooltip: String,
) -> Element {
    let color = if selected { color } else { "#666".to_string() };
    use dioxus_free_icons::Icon;
    rsx! {
        div {
            class: "icon-container",
            onclick: move |_| {
                onclick.call(());
            },
            div {
                class: "icon-box",
                "data-tooltip": "{tooltip}",
                "data-placement": "top",
                Icon {
                    width: 26,
                    height: 26,
                    fill: "{color}",
                    icon: icon,
                }
            }
        }
    }
}

use dioxus::prelude::*;
use dioxus_free_icons::IconShape;


#[component]
pub fn Icon<T: IconShape + Clone + PartialEq + 'static>
 (icon: T, color: String, selected: bool, onclick: Callback<()> , tooltip: String) -> Element {
    let color = if selected { color } else {"#666".to_string()};
    use dioxus_free_icons::Icon;
    rsx! {
        div {
            style: "
                height: 100%;
                padding: 4px; margin: 4px;
                flex-grow: 1;
            ",
            onclick: move |_| {
                onclick.call(());
            },
            cursor: "pointer",
            div {
                style: "
                width: 100%;
                height: 46px;
                margin: auto;
                ",
                "data-tooltip": "{tooltip}",
                "data-placement": "top",
                cursor: "pointer",
                div {
                    style: "
                    height: 36px;
                    width: 36px;
                    margin: auto;
                    ",
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
}

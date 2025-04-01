use dioxus::prelude::*;
use dioxus_free_icons::IconShape;


#[component]
pub fn Icon<T: IconShape + Clone + PartialEq + 'static>
 (icon: T, color: String) -> Element {
    use dioxus_free_icons::Icon;
    rsx! {
        div {
            style: "
            // width: 38px;
            height: 38px;
            // border: 1px solid {color};
            padding: 4px; margin: 4px;
            flex-grow: 1;
            ",
            div {
                style: "
                width: 100%;
                height: 100%;
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

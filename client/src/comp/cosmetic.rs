use dioxus::prelude::*;

#[component]
pub fn Hline() -> Element {
    rsx! {
        HlineInner {  }
        HlineInner {  }
    }
}

#[component]
pub fn HlineInner() -> Element {
    rsx! {
        div {
            style: "
            width: 100%;
            height: 1px;
            border: 1px solid black;
            padding: 0px;
            margin: 1px;
            "
        }
    }
}

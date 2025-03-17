use dioxus::prelude::*;
#[component]
pub fn ModalArticle(on_close: Callback<()>, title: Element, body: Element) -> Element {
    rsx! {
        dialog {
            open: true,

            onclick: move |_| {
                on_close.call(());
            },
            article {
                onclick: move |_e| {
                    _e.stop_propagation();
                },
                header {
                    button {
                        "aria-label": "Close",
                        "rel": "prev",
                        onclick: move |_| {
                            on_close.call(());
                        },
                        ""
                    }
                    p {
                        strong {
                            {title}
                        }
                    }
                }
                {body}
            }
        }
    }
}

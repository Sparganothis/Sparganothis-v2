use dioxus::prelude::*;
use crate::route::Route;


/// Home page
#[component]
pub fn Home() -> Element {
    rsx! {
        article {
            header {
                "default header"
            }

            "Body"

            footer {
                "default footer"
            }
        }

    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            // Navigation links
            Link {
                to: Route::Blog { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                "Next"
            }
        }
    }
}
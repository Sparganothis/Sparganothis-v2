use dioxus::prelude::*;

use crate::comp::icon::Icon;

#[component]
pub fn MiniChatOverlay () -> Element {
    rsx! {
        article {
            id: "mini_chat_overlay",
            style: "
            position: absolute;
            right: 1.5rem;
            bottom: 1.5rem;
            padding: 0.5rem;
            margin: 0.5rem;
            width: 350px;
            height: 450px;
            border: 3px dashed blue;
            z-index: 2;
            background-color: white;
            ",
            MiniChatLayout {
                div {
                    "content2"
                }
            }
        }
    }
}

pub enum MiniChatTabSelection {

}

#[component]
pub fn MiniChatLayout (children: Element) -> Element {
    rsx! {
        div {
            style: r#"
            display: grid; 
            grid-template-columns: 1fr; 
            grid-template-rows: 0.3fr 1.9fr 0.3fr; 
            gap: 0px 0px; 
            grid-template-areas: "topbar"   "mainchat"  "tabs"; 
            width: 100%;
            height: 100%;
            "#,
            
            div {
                style: "
                grid-area: topbar;
                width: 100%;
                height: 100%;
                border: 1px solid green;
                container-type: size;
                ",

                MiniChatTopBar {}
            }
            div {
                style: "
                grid-area: mainchat;
                width: 100%;
                height: 100%;
                border: 1px solid red;
                container-type: size;
                ",

                "content",
                {children}
            }
            div {
                style: " 
                grid-area: tabs;
                width: 100%;
                height: 100%;
                border: 1px solid blue;
                container-type: size;
                ",
                "tabs"
            }
        }
    }
}


#[component]
fn MiniChatTopBar () -> Element {
use dioxus_free_icons::icons::bs_icons::*;

    rsx! {
        div {
            style: "
            display: flex;
            justify-content: center;
            align-items: center;
            ",
            
            Icon {
                icon:  BsChatRightText,
                color: "green",
            }
            Icon {
                icon: BsPersonLinesFill,
                color: "blue",
            }
            Icon {
                icon:  BsXLg,
                color: "red",
            }
        }
    }
}

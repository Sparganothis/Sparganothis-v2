use dioxus::prelude::*;

const INTER_BOX_PADDING : &'static str = "0.1cqmin";

#[component]
pub fn GameDisplay() -> Element {
    rsx! {
        div {
            style: "
                width: 100%;
                height: 80dvh;
                display: flex;
                align-items: center;
                justify-content: center;
                border: 1px solid blue;
                container-type:size;
            ",

            div {
                style: "
                width: calc(min(100cqw, 50cqh));
                height: calc(min(100cqh, min(100cqh, 200cqw)));
                border: 1px solid yellow;
                display: flex;
                align-items: center;
                justify-content: center;
                border: 1px solid green;
                ",
                            
                div {
                    style: "
                        border: 1px solid red;
                        padding: 1px;
                        width: 100%;
                        height: 100%;
                        border: 0.1vmin solid  rgba(95, 67, 33, 0.44);
                        container-type:size;
                    ",
                    GameDisplayInner{}
                }
            }

        }
    }
}

#[component]
fn GameDisplayInner() -> Element {
    rsx! {
        GameBoardDisplay{}
    }
}

#[component]
fn GameBoardDisplay() -> Element {
    let column_count = 10;
    let row_count = 20;

    rsx! {
        div {
            style: "
                display: grid;
                grid-template-columns: repeat({column_count}, minmax(0, 1fr));
                grid-template-rows: repeat({row_count}, auto);
                grid-column-gap: 0px;
                grid-row-gap: 0px;
                width: 100%;
                height: 100%;
                background-color:rgba(95, 67, 33, 0.44);
                padding: 0.1vmin;
            ",

            for _row_id in 0..row_count {
                for _col_id in 0..column_count {
                    GridCell {}
                }
            }
        }
    }
}

#[component]
fn GridCell() -> Element {
    rsx! {
        div {
            style: "
                width: calc(100cqw/10-0.1vmin);
                height: calc(100cqh/20-0.1vmin);
                padding: 0.1vmin;
            ",
            div {
                style: "
                border: 0.1vmin solid rgba(95, 67, 33, 0.44); 
                background-color: rgba(17, 37, 78, 0.54);
                width: 100%;
                height: 100%;
                aspect-ratio: 1/1;
                "
            }
        }
    }
}
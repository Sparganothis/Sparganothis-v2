use dioxus::prelude::*;

const INTER_BOX_PADDING : &'static str = "0.1cqmin";
const GAMEBOARD_GRID_COLOR : &'static str = "rgba(109, 20, 20, 0.44)";
const GAMEBOARD_BOX_EMPTY_COLOR : &'static str = "rgba(28, 16, 99, 0.54)";

#[component]
pub fn GameDisplay() -> Element {
    rsx! {
        div {
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                align-items: center;
                justify-content: center;
                container-type:size;
            ",

            div {
                style: "
                width: calc(min(100cqw, 50cqh));
                height: calc(min(100cqh, min(100cqh, 200cqw)));
                border: {INTER_BOX_PADDING} solid yellow;
                display: flex;
                align-items: center;
                justify-content: center;
                border: {INTER_BOX_PADDING} solid {GAMEBOARD_GRID_COLOR};
                ",
                            
                div {
                    style: "
                        border: {INTER_BOX_PADDING} solid red;
                        padding: {INTER_BOX_PADDING};
                        width: 100%;
                        height: 100%;
                        border: {INTER_BOX_PADDING} solid  {GAMEBOARD_GRID_COLOR};
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
                background-color:{GAMEBOARD_GRID_COLOR};
                padding: {INTER_BOX_PADDING};
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
                width: calc(100cqw/10-{INTER_BOX_PADDING});
                height: calc(100cqh/20-{INTER_BOX_PADDING});
                padding: {INTER_BOX_PADDING};
            ",
            div {
                style: "
                border: {INTER_BOX_PADDING} solid {GAMEBOARD_GRID_COLOR}; 
                background-color: {GAMEBOARD_BOX_EMPTY_COLOR};
                width: 100%;
                height: 100%;
                aspect-ratio: 1/1;
                "
            }
        }
    }
}
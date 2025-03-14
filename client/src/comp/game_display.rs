use dioxus::prelude::*;
use game::tet::{CellValue, GameState, Tet};

const INTER_BOX_PADDING: &'static str = "0px";
const GAMEBOARD_GRID_COLOR: &'static str = "rgb(0, 0, 0)";

// Add this function to map Tet to color
fn get_tet_color(tet: &Tet) -> &'static str {
    match tet {
        Tet::I => "rgb(40, 218, 182)", // Cyan
        Tet::J => "rgb(26, 53, 196)",   // Blue
        Tet::L => "rgb(223, 131, 20)", // Orange
        Tet::O => "rgb(228, 196, 15)", // Yellow
        Tet::S => "rgb(31, 204, 89)",   // Green
        Tet::T => "rgb(199, 41, 151)", // Purple
        Tet::Z => "rgb(197, 57, 32)",   // Red
    }
}

fn get_cell_color(cell_value: Option<CellValue>) -> &'static str {
    match cell_value {
        Some(CellValue::Piece(tet)) => get_tet_color(&tet),
        Some(CellValue::Ghost) => "rgb(109, 109, 109)",
        Some(CellValue::Garbage) => "rgba(128, 128, 128, 0.8)",
        _ => "rgb(26, 26, 26)",
    }
}

#[component]
pub fn GameDisplay(game_state: ReadOnlySignal<GameState>) -> Element {
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
            
            GameDetailsLeftPane {
                game_state: game_state
            }

            div {
                style: "
                width: calc(min(100cqw, 50cqh));
                height: calc(min(100cqh, min(100cqh, 200cqw)));
                display: flex;
                align-items: center;
                justify-content: center;
                ",

                div {
                    style: "
                        padding: {INTER_BOX_PADDING};
                        width: 100%;
                        height: 100%;
                        container-type:size;
                        border: 3px dashed red;
                        display: flex;
                    ",
                    GameBoardDisplay{
                        game_state: game_state
                    }
                }
            }
            
            GameDetailsRightPane {
                game_state: game_state
            }
        }
    }
}

#[component]
fn GameDetailsLeftPane(game_state: ReadOnlySignal<GameState>) -> Element {
    rsx! {
        div {
            style: "
                width: 300px;
                height: 100%;
                border: 3px dashed blue;
            ",
        }
    }
}

#[component]
fn GameDetailsRightPane(game_state: ReadOnlySignal<GameState>) -> Element {
    rsx! {
        div {
            style: "
                width: 300px;
                height: 100%;
                border: 3px dashed green;
            ",
        }
    }
}




#[component]
fn GameBoardDisplay(game_state: ReadOnlySignal<GameState>) -> Element {
    let column_count = 10;
    let row_count = 20;

    rsx! {
        div {
            style: "
                position: relative;
                display: grid;
                grid-template-columns: repeat({column_count}, minmax(0, 1fr));
                grid-template-rows: repeat({row_count}, auto);
                grid-column-gap: 0px;
                grid-row-gap: 0px;
                width: 100%;
                height: 100%;
                background-color:{GAMEBOARD_GRID_COLOR};
                padding: {INTER_BOX_PADDING};
                border: 1px solid {GAMEBOARD_GRID_COLOR};
            ",

            for row_id in 0..row_count {
                for col_id in 0..column_count {
                    GridCell {
                        game_state: game_state,
                        row: (row_count - 1 - row_id) as i8,
                        col: col_id as i8
                    }
                }
            }
        }
    }
}

#[component]
fn GridCell(game_state: ReadOnlySignal<GameState>, row: i8, col: i8) -> Element {
    let cell_value = use_memo(move || game_state.read().main_board.get_cell(row, col));
    let cell_color = use_memo(move || get_cell_color(cell_value.read().clone()));

    rsx! {
        div {
            style: "
                posiition: absolute;
                width: calc(100cqw/10-{INTER_BOX_PADDING});
                height: calc(100cqh/20-{INTER_BOX_PADDING});
                top: calc((100cqh/20-{INTER_BOX_PADDING}) * {row});
                left: calc((100cqw/10-{INTER_BOX_PADDING}) * {col});
                padding: {INTER_BOX_PADDING};
            ",
            div {
                style: "
                background-color: {cell_color};
                width: 100%;
                height: 100%;
                aspect-ratio: 1/1;
                border: 1px solid {GAMEBOARD_GRID_COLOR};
                "
            }
        }
    }
}

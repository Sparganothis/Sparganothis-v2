use dioxus::prelude::*;
use game::tet::{BoardMatrix, BoardMatrixNext, CellValue, GameState, Tet, TetAction};

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

            div {
                style: " 
                    width: 30%;
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                ",
            
            }

            div {
                style: " 
                    width: 40%;
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                ",

                GameDisplayInner {
                    game_state: game_state
                }
            }

            div {
                style: " 
                    width: 30%;
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                ",

            }
        }
    }
}

#[component]
fn GameDetailsLeftPane(game_state: ReadOnlySignal<GameState>) -> Element {
    let hold_press = use_memo(move || game_state.read().last_action == TetAction::Hold);
    let hold_color = use_memo(move || if *hold_press.read() { "rgb(128, 0, 0)" } else { "rgb(0, 0, 0)" });
    rsx! {
        div {
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                align-items: start;
                justify-content: end;
            ",
            div {
                style: "
                    width: 50%;
                    font-size: 15cqmin;
                    font-weight: bold;
                    text-align: center;
                    color: {hold_color};
                ",
                "Hold"
            }
            div {
                style: "
                    width: 50%;
                    align:right;
                ",
                GameBoardDisplayHoldGrid {
                    game_state: game_state
                }
            }
        }
    }
}

#[component]
fn GameDetailsRightPane(game_state: ReadOnlySignal<GameState>) -> Element {
    let next_press = use_memo(move || game_state.read().last_action == TetAction::HardDrop );
    let next_color = use_memo(move || if *next_press.read() { "rgb(128, 0, 0)" } else { "rgb(0, 0, 0)" });
    rsx! {
        div {
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                align-items: start;
                justify-content: start;
            ",
            div {
                style: "
                    width: 50%;
                    margin-bottom: auto;
                    align:left;
                ",
                GameBoardDisplayNextGrid {
                    game_state: game_state
                }
            }
            div {
                style: "
                    width: 50%;
                    font-size: 15cqmin;
                    font-weight: bold;
                    text-align: center;
                    color: {next_color};
                ",
                "Next"
            }
        }
    }
}


#[component]
fn GameDisplayInner(game_state: ReadOnlySignal<GameState>) -> Element {
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
                    position: relative;
                    width: calc(min(100cqw, 50cqh));
                    height: calc(min(100cqh, min(100cqh, 200cqw)));
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    container-type:size;
                ",
                div {
                    style: "
                    position:absolute;
                        top: 0; 
                        left: -74cqw;
                        width: 73cqw;
                        height: 99cqh;
                    ",
                    GameDetailsLeftPane {
                        game_state: game_state
                    }
                }

                div {
                    style: "
                        padding: {INTER_BOX_PADDING};
                        width: 100%;
                        height: 100%;
                        container-type:size;
                        display: flex;
                    ",
                    GameBoardDisplayMainGrid{
                        game_state: game_state
                    }
                }
                div {
                    style: "
                    position:absolute;
                        top: 0; 
                        left: 101cqw;
                        width: 73cqw;
                        height: 99cqh;
                    ",
                    GameDetailsRightPane {
                        game_state: game_state
                    }
                }
            }

        }
    }
}


#[component]
fn GameBoardDisplayMainGrid(game_state: ReadOnlySignal<GameState>) -> Element {
    let main_board = use_memo(move || game_state.read().main_board);
    rsx! {
        BoardGrid {
            board: main_board
        }
    }
}

#[component]
fn GameBoardDisplayNextGrid(game_state: ReadOnlySignal<GameState>) -> Element {
    let next_board = use_memo(move || game_state.read().get_next_board());
    rsx! {
        BoardGrid {
            board: next_board
        }
    }
}
#[component]
fn GameBoardDisplayHoldGrid(game_state: ReadOnlySignal<GameState>) -> Element {
    let hold_board = use_memo(move || game_state.read().get_hold_board());
    rsx! {
        BoardGrid {
            board: hold_board
        }
    }
}


#[component]
fn BoardGrid<const R: usize, const C: usize>(board: ReadOnlySignal<BoardMatrix<R, C>>) -> Element {
    let column_count = C as i8;
    let row_count = (R as i8).min(20);
    rsx! {
        GameBoardGridParent {
            column_count: column_count,
            row_count: row_count,
            children: rsx! {
                for row_id in 0..row_count {
                    for col_id in 0..column_count {
                        BoardGridCell {
                            board: board,
                            row: (row_count - 1 - row_id) as i8,
                            col: col_id as i8,
                            row_count: row_count,
                            col_count: column_count
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BoardGridCell<const R: usize, const C: usize>(board: ReadOnlySignal<BoardMatrix<R, C>>, row: i8, col: i8, row_count: i8, col_count: i8) -> Element {
    let cell = use_memo(move || board.read().get_cell(row, col));
    rsx! {
        GridCellDisplay {
            cell: cell,
            row: row,
            col: col,
            row_count: row_count,
            col_count: col_count
        }
    }
}

#[component]
fn GameBoardGridParent(column_count: i8, row_count: i8, children: Element) -> Element {
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
                aspect-ratio: {column_count}/{row_count};
            ",
            {children}
        }
    }
}

#[component]
fn GridCellDisplay(cell: ReadOnlySignal<Option<CellValue>>, row: i8, col: i8, row_count: i8, col_count: i8) -> Element {
    let cell_color = use_memo(move || get_cell_color(cell.read().clone()));
    rsx! {
        div {
            style: "
                posiition: absolute;
                width: calc(100cqw/{col_count}-{INTER_BOX_PADDING});
                height: calc(100cqh/{row_count}-{INTER_BOX_PADDING});
                top: calc((100cqh/{row_count}-{INTER_BOX_PADDING}) * {row});
                left: calc((100cqw/{col_count}-{INTER_BOX_PADDING}) * {col});
                padding: {INTER_BOX_PADDING};
            ",
            div {
                style: "
                background-color: {cell_color};
                width: 100%;
                height: 100%;
                aspect-ratio: 1/1;
                border: 0.1cqmin solid {GAMEBOARD_GRID_COLOR};
                "
            }
        }
    }
}

use dioxus::prelude::*;
use game::tet::{BoardMatrix, CellValue, GameOverReason, GameState, Tet};

// const INTER_BOX_PADDING: &'static str = "0px";
const GAMEBOARD_GRID_COLOR: &'static str = "rgb(0, 0, 0)";

// Add this function to map Tet to color
fn get_tet_color(tet: &Tet) -> &'static str {
    match tet {
        Tet::I => "rgb(40, 218, 182)", // Cyan
        Tet::J => "rgb(26, 53, 196)",  // Blue
        Tet::L => "rgb(223, 131, 20)", // Orange
        Tet::O => "rgb(228, 196, 15)", // Yellow
        Tet::S => "rgb(31, 204, 89)",  // Green
        Tet::T => "rgb(199, 41, 151)", // Purple
        Tet::Z => "rgb(197, 57, 32)",  // Red
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
pub fn YouDied(
    game_state: ReadOnlySignal<GameState>,
    children: Element,
) -> Element {
    let msg = use_memo(move || {
        let r = game_state.read().clone().game_over_reason;
        match r {
            None => "",
            Some(GameOverReason::Win) => "YOU WIN",
            Some(GameOverReason::Knockout) => "K.O.",
            Some(GameOverReason::Disconnect) => "DISCONNECT",
            Some(GameOverReason::Abandon) => "ABANDON",
        }
        .to_string()
    });
    let color = use_memo(move || {
        let r = game_state.read().clone().game_over_reason;
        match r {
            None => "",
            Some(GameOverReason::Win) => "green",
            Some(GameOverReason::Knockout) => "red",
            Some(GameOverReason::Disconnect) => "orange",
            Some(GameOverReason::Abandon) => "purple",
        }
        .to_string()
    });
    rsx! {
        if game_state.read().game_over() {
            div { class: "game-overlay",
                h3 { 
                    class: "text-center",
                    style: "color: {color}; font-size: 5rem; text-shadow: 0 0 20px rgba(0,0,0,0.5);",
                    "{msg}"
                }
            }
        }
        {children}
    }
}

#[component]
pub fn GameDisplay(game_state: ReadOnlySignal<GameState>) -> Element {
    rsx! {
        div { class: "flex items-center justify-center w-full h-full",
            YouDied {
                game_state,
                GameDisplayInner { game_state }
            }
        }
    }
}

#[component]
fn GameDetailsLeftPane(game_state: ReadOnlySignal<GameState>) -> Element {
    rsx! {
        div { class: "flex flex-col items-end justify-start gap-2 w-full h-full",
            div { class: "w-full text-right",
                GameBoardDisplayHoldGrid { game_state }
            }
            GameStateInfo { game_state }
        }
    }
}

#[component]
fn GameDetailsRightPane(game_state: ReadOnlySignal<GameState>) -> Element {
    rsx! {
        div { class: "flex items-start justify-start w-full h-full",
            div { class: "w-full mb-auto text-left",
                GameBoardDisplayNextGrid { game_state }
            }
        }
    }
}

#[component]
fn GameDisplayInner(game_state: ReadOnlySignal<GameState>) -> Element {
    rsx! {
        div { class: "flex items-center justify-center w-full h-full",
            div { 
                class: "flex items-center justify-center",
                style: "position: relative; width: calc(min(100cqw, 50cqh)); height: calc(min(100cqh, min(100cqh, 200cqw)));",
                div { 
                    class: "h-full",
                    style: "position:absolute; top: 0; left: -74cqw; width: 73cqw;",
                    GameDetailsLeftPane { game_state }
                }

                div { class: "w-full h-full flex",
                    GameBoardDisplayMainGrid { game_state }
                }
                div { 
                    class: "h-full",
                    style: "position:absolute; top: 0; left: 101cqw; width: 73cqw;",
                    GameDetailsRightPane { game_state }
                }
            }
        }
    }
}

#[component]
fn GameBoardDisplayMainGrid(game_state: ReadOnlySignal<GameState>) -> Element {
    let main_board = use_memo(move || game_state.read().main_board);
    rsx! {
        BoardGrid { board: main_board }
    }
}

#[component]
fn GameBoardDisplayNextGrid(game_state: ReadOnlySignal<GameState>) -> Element {
    let next_board = use_memo(move || game_state.read().get_next_board());
    rsx! {
        BoardGrid { board: next_board }
    }
}
#[component]
fn GameBoardDisplayHoldGrid(game_state: ReadOnlySignal<GameState>) -> Element {
    let hold_board = use_memo(move || game_state.read().get_hold_board());
    rsx! {
        BoardGrid { board: hold_board }
    }
}

#[component]
fn BoardGrid<const R: usize, const C: usize>(
    board: ReadOnlySignal<BoardMatrix<R, C>>,
) -> Element {
    let column_count = C as i8;
    let row_count = (R as i8).min(20);
    rsx! {
        GameBoardGridParent {
            column_count,
            row_count,
            children: rsx! {
                for row_id in 0..row_count {
                    for col_id in 0..column_count {
                        BoardGridCell {
                            board,
                            row: (row_count - 1 - row_id) as i8,
                            col: col_id as i8,
                            row_count,
                            col_count: column_count,
                        }
                    }
                }
            },
        }
    }
}

#[component]
fn BoardGridCell<const R: usize, const C: usize>(
    board: ReadOnlySignal<BoardMatrix<R, C>>,
    row: i8,
    col: i8,
    row_count: i8,
    col_count: i8,
) -> Element {
    let cell = use_memo(move || board.read().get_cell(row, col));
    rsx! {
        GridCellDisplay {
            cell,
            row,
            col,
            row_count,
            col_count,
        }
    }
}

#[component]
fn GameBoardGridParent(
    column_count: i8,
    row_count: i8,
    children: Element,
) -> Element {
    rsx! {
        div { 
            class: "game-board-wrapper",
            style: "
                display: grid;
                grid-template-columns: repeat({column_count}, minmax(0, 1fr));
                grid-template-rows: repeat({row_count}, auto);
                aspect-ratio: {column_count}/{row_count};
            ",
            {children}
        }
    }
}

#[component]
fn GridCellDisplay(
    cell: ReadOnlySignal<Option<CellValue>>,
    row: i8,
    col: i8,
    row_count: i8,
    col_count: i8,
) -> Element {
    let cell_color = use_memo(move || get_cell_color(cell.read().clone()));
    rsx! {
        div { class: "p-0",
            div { 
                class: "game-cell",
                style: "background-color: {cell_color};",
            }
        }
    }
}

#[component]
fn GameStateInfo(game_state: ReadOnlySignal<GameState>) -> Element {
    let state = game_state.read();
    rsx! {
        div {
            id: "game-state-info",
            class: "game-stats-list w-full text-right p-2",
            
            div { class: "game-stat-entry",
                span { class: "game-stat-label", "Score" }
                span { class: "game-stat-value", "{state.score}" }
            }
            div { class: "game-stat-entry",
                span { class: "game-stat-label", "Lines" }
                span { class: "game-stat-value", "{state.total_lines}" }
            }
            div { class: "game-stat-entry",
                span { class: "game-stat-label", "Moves" }
                span { class: "game-stat-value", "{state.total_moves}" }
            }
            div { class: "game-stat-entry",
                span { class: "game-stat-label", "Combo" }
                span { class: "game-stat-value", "{state.combo_counter}" }
            }
            div { class: "game-stat-entry",
                span { class: "game-stat-label", "Time" }
                span { class: "game-stat-value", "{state.current_time_string()}" }
            }

            // Show B2B and T-spin indicators if active
            if state.is_b2b {
                div { class: "text-gold mt-1", "Back-to-Back!" }
            }
            if state.is_t_spin {
                div { class: "text-pink mt-1", "T-Spin!" }
            }
            if state.garbage_recv > 0 {
                div { class: "text-red mt-1", "Incoming: {state.garbage_recv - state.garbage_applied}" }
            }
        }
    }
}

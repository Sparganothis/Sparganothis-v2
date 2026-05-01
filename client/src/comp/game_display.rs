use dioxus::prelude::*;
use game::tet::{BoardMatrix, CellValue, GameOverReason, GameState, Tet};
use rand::Rng;

// const INTER_BOX_PADDING: &'static str = "0px";
const GAMEBOARD_GRID_COLOR: &'static str = "rgba(0, 0, 0, 0)";

// Add this function to map Tet to color
fn get_tet_color(tet: &Tet) -> RgbaColor {
    match tet {
        Tet::I => RgbaColor::rgb(40, 218, 182), // Cyan
        Tet::J => RgbaColor::rgb(26, 53, 196),  // Blue
        Tet::L => RgbaColor::rgb(223, 131, 20), // Orange
        Tet::O => RgbaColor::rgb(228, 196, 15), // Yellow
        Tet::S => RgbaColor::rgb(31, 204, 89),  // Green
        Tet::T => RgbaColor::rgb(199, 41, 151), // Purple
        Tet::Z => RgbaColor::rgb(197, 57, 32),  // Red
    }
}

#[derive(Clone, PartialEq, Copy)]
struct RgbaColor {
    r: u8,
    g: u8,
    b: u8,
    a: f32,
}

impl RgbaColor {
    fn rgb(r: u8, g: u8, b: u8) -> RgbaColor {
        RgbaColor {
            r,g,b,a: 1.0,
        }
    }
    fn rgba(r: u8, g: u8, b: u8, a: f32) -> RgbaColor {
        RgbaColor {
            r,g,b,a,
        }
    }
    fn string(&self) -> String {
        format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a)
    }
    fn darken(&self) -> Self {
        let Self{r,g,b,a} = *self;
        RgbaColor {
            r: (r as f32 * 0.9) as u8,
            g: (g as f32 * 0.9) as u8,
            b: (b as f32 * 0.9) as u8,
            a: a * 0.8_f32,
        }
    }
}

fn get_cell_color(cell_value: Option<CellValue>) -> RgbaColor {
    match cell_value {
        Some(CellValue::Piece(tet)) => get_tet_color(&tet),
        Some(CellValue::Ghost) => RgbaColor::rgb(109, 109, 109),
        Some(CellValue::Garbage) => RgbaColor::rgba(128, 128, 128, 0.8_f32),
        _ => RgbaColor::rgb(26, 26, 26)
    }
}

#[component]
pub fn YouDied(
    game_state: ReadSignal<GameState>,
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
            div {
                style: "position: relative; width: 0; height: 0; margin: 0; padding: 0; top: 0px; left: 0px;",
                div {
                    style: "position: absolute; width: 20cqw; height: 20cqh; color: red; z-index: 666;",
                    h3 {
                        style: "color:{color}; z-index: 666; font-size: 6rem; transform: rotate(-45deg); background-color: black; width: fit-content; height: fit-content;",
                        "{msg}"
                    }
                }
            }
        }
        {children}
    }
}

#[component]
pub fn GameDisplay(game_state: ReadSignal<GameState>) -> Element {
    rsx! {
        div { style: "
                width: 100%;
                height: 100%;
                display: flex;
                align-items: center;
                justify-content: center;
                container-type:size;

                
                background-color: rgba(0,0,0,0.3);
            ",

            div { style: "
                    width: 30%;
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                " }

            div { style: "
                    width: 40%;
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                ",

                YouDied {
                    game_state,
                    GameDisplayInner { game_state }
                }
            }

            div { style: "
                    width: 30%;
                    height: 100%;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                " }
        }
    }
}

#[component]
fn GameDetailsLeftPane(game_state: ReadSignal<GameState>) -> Element {
    rsx! {
        div { style: "
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                align-items: end;
                justify-content: start;
                gap: 20px;
            ",
            div { style: "
                    width: 50%;
                    align:right;
                ",
                GameBoardDisplayHoldGrid { game_state }
            }

            GameStateInfo { game_state }
        }
    }
}

#[component]
fn GameDetailsRightPane(game_state: ReadSignal<GameState>) -> Element {
    rsx! {
        div { style: "
                width: 100%;
                height: 100%;
                display: flex;
                align-items: start;
                justify-content: start;
            ",
            div { style: "
                    width: 50%;
                    margin-bottom: auto;
                    align:left;
                ",
                GameBoardDisplayNextGrid { game_state }
            }

        }
    }
}

#[component]
fn GameDisplayInner(game_state: ReadSignal<GameState>) -> Element {
    rsx! {
        div { style: "
                width: 100%;
                height: 100%;
                display: flex;
                align-items: center;
                justify-content: center;
                container-type:size;
            ",
            div { style: "
                    position: relative;
                    width: calc(min(100cqw, 50cqh));
                    height: calc(min(100cqh, min(100cqh, 200cqw)));
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    container-type:size;
                ",
                div { style: "
                    position:absolute;
                        top: 0; 
                        left: -74cqw;
                        width: 73cqw;
                        height: 99cqh;
                    ",
                    GameDetailsLeftPane { game_state }
                }

                div { style: "
                        padding: 0px;
                        width: 100%;
                        height: 100%;
                        container-type:size;
                        display: flex;
                    ",
                    GameBoardDisplayMainGrid { game_state }
                }
                div { style: "
                    position:absolute;
                        top: 0; 
                        left: 101cqw;
                        width: 73cqw;
                        height: 99cqh;
                    ",
                    GameDetailsRightPane { game_state }
                }
            }

        }
    }
}

#[component]
fn GameBoardDisplayMainGrid(game_state: ReadSignal<GameState>) -> Element {
    let main_board = use_memo(move || game_state.read().main_board);
    rsx! {
        BoardGrid { board: main_board }
    }
}

#[component]
fn GameBoardDisplayNextGrid(game_state: ReadSignal<GameState>) -> Element {
    let next_board = use_memo(move || game_state.read().get_next_board());
    rsx! {
        BoardGrid { board: next_board }
    }
}
#[component]
fn GameBoardDisplayHoldGrid(game_state: ReadSignal<GameState>) -> Element {
    let hold_board = use_memo(move || game_state.read().get_hold_board());
    rsx! {
        BoardGrid { board: hold_board }
    }
}

#[component]
fn BoardGrid<const R: usize, const C: usize>(
    board: ReadSignal<BoardMatrix<R, C>>,
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
    board: ReadSignal<BoardMatrix<R, C>>,
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
        div { style: "
                position: relative;
                display: grid;
                grid-template-columns: repeat({column_count}, minmax(0, 1fr));
                grid-template-rows: repeat({row_count}, auto);
                grid-column-gap: 0px;
                grid-row-gap: 0px;
                width: 100%;
                height: 100%;
                background-color:{GAMEBOARD_GRID_COLOR};
                padding: 0px;
                border: 1px solid {GAMEBOARD_GRID_COLOR};
                aspect-ratio: {column_count}/{row_count};
            ",
            {children}
        }
    }
}

#[component]
fn GridCellDisplay(
    cell: ReadSignal<Option<CellValue>>,
    row: i8,
    col: i8,
    row_count: i8,
    col_count: i8,
) -> Element {
    let cell_color = use_memo(move || get_cell_color(cell.read().clone()));
    let rot_deg = use_memo(move || {
        let rng = &mut rand::thread_rng();
        rng.gen_range(0.0_f32..360.0_f32)
        
    });

    let cell1 = cell_color.read().darken().string();
    let cell2 = cell_color.read().string();
    let rot_deg = rot_deg();

    let mut cell3 = cell_color.read().darken().darken().darken().darken().darken();
    cell3.a *= 0.3;
    
    let cell3 = cell3.string();
    let mut cell4 = cell_color.read().clone();
    cell4.a *= 0.25;
    let cell4 = cell4.string();


    let zindex = use_memo(move || {
        let c = cell.read().clone();
        match c {
            None | Some(CellValue::Empty) => {
                78
            }
            Some(_x) => {
                6
            }
        }
    });
    let shadow =  {
        match  cell.read().clone() {
            None | Some(CellValue::Empty) => {
                format!("")
            }
            Some(_x) => {
      
                        format!(" box-shadow: 
                        inset 0cqmin 0cqmin  0.5cqmin  0.5cqmin {cell3}, 
                        0cqmin 0cqmin  37cqmin  0cqmin {cell4};")
            }
        }
    };


    //         position: absolute;
    // width: calc(100cqw/{col_count});
    // height: calc(100cqh/{row_count});
    // top: calc((100cqh/{row_count}) * {row});
    // left: calc((100cqw/{col_count}) * {col});


    
                // background-color: linear-gradient({rot_deg}deg, {cell1}, 
// 
    rsx! {
        div { style: "
                padding: 0;
            ",
            div { style: "
                 background-image: linear-gradient({rot_deg}deg, {cell2}, {cell1});
                 
                {shadow}
                width: 100%;
                height: 100%;
                aspect-ratio: 1/1;
                border: 0.5cqmin solid {GAMEBOARD_GRID_COLOR};
                z-index: {zindex};
                ",
            }
        }
    }
}

#[component]
fn GameStateInfo(game_state: ReadSignal<GameState>) -> Element {
    let state = game_state.read();
    rsx! {
        div {
            id: "game-state-info",

            style: "
                width: 100%;
                font-family: monospace;
                font-size: 1.2em;
                text-align: right;
                padding-right: 20px;
                color: black;
            ",
            div { "Score: {state.score}" }
            div { "Lines: {state.total_lines}" }
            div { "Moves: {state.total_moves}" }
            div { "Combo: {state.combo_counter}" }
            div { "Time: {state.current_time_string()}" }
            div { "Lines Sent: {state.total_garbage_sent}"}
            div { "Lines Recv: {state.garbage_recv}"}
            div { "Lines Applied: {state.garbage_applied}"}


            // Show B2B and T-spin indicators if active
            if state.is_b2b {
                div { style: "color: #ffd700;", // Gold color for special states
                    "Back-to-Back!"
                }
            }

            if state.is_t_spin {
                div { style: "color: #ff69b4;", // Pink color for T-spin
                    "T-Spin!"
                }
            }

            // Show garbage info if any
            if state.garbage_recv > 0 {
                div { style: "color: #ff4444;", // Red color for garbage
                    "Incoming: {state.garbage_recv - state.garbage_applied}"
                }
            }
        }
    }
}

mod game_state;
mod matrix;
mod random;
mod rot;
mod tetpcs;

pub use game_state::{
    segments_to_states, CurrentPcsInfo, GameOverReason, GameReplaySegment,
    GameReplaySlice, GameState, HoldPcsInfo,
};
pub use matrix::{BoardMatrix, BoardMatrixHold, BoardMatrixNext, CellValue};
pub use random::{get_random_seed, GameSeed};
pub use rot::RotState;
pub use tetpcs::{Tet, TetAction};

#[cfg(test)]
pub mod tests {
    use super::super::timestamp::get_timestamp_now_ms;
    use super::*;
    use game_state::{GameReplaySegment, GameState};
    use tetpcs::TetAction;
    // use pretty_assertions::assert_eq;
    use wasm_bindgen_test::*;

    #[test]
    #[wasm_bindgen_test]
    pub fn random_have_pinned_results() {
        let seed = [0; 32];
        let mut state = GameState::new(&seed, 0);

        // let expected_seed = [0;32];
        // assert_eq!(expected_seed, state.seed);

        state
            .apply_action_if_works(TetAction::UserSoftDrop, 0)
            .unwrap();

        let expected_seed = [128, 238, 116, 255, 23, 240, 86, 11, 28, 25, 139, 43, 63, 116, 75, 203, 47, 156, 66, 89, 6, 77, 240, 2, 102, 224, 139, 30, 5, 160, 15, 152];
        assert_eq!(state.seed, expected_seed,);

        state.apply_action_if_works(TetAction::HardDrop, 1).unwrap();

        let expected_seed = [94, 83, 37, 193, 117, 132, 21, 152, 13, 195, 65, 23, 121, 22, 121, 252, 99, 153, 227, 21, 17, 60, 44, 24, 247, 17, 123, 213, 24, 135, 148, 59];
        assert_eq!(expected_seed, state.seed);
    }

    #[test]
    #[wasm_bindgen_test]
    #[allow(clippy::unnecessary_unwrap)]
    pub fn active_game_is_deterministic() {
        for i in 0..255 {
            let seed = [i; 32];
            let mut state1 = GameState::new(&seed, get_timestamp_now_ms());
            let mut state2 = GameState::new(&seed, state1.start_time);

            loop {
                let action = TetAction::random();
                let t2 = get_timestamp_now_ms();
                let res1 = state1.try_action(action, t2).map_err(|_| "bad");
                let res2 = state2.try_action(action, t2).map_err(|_| "bad");
                assert_eq!(res1, res2);
                if res1.is_ok() {
                    state1 = res1.unwrap();
                    state2 = res2.unwrap();
                }

                if state1.game_over() {
                    break;
                }
            }
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn passive_game_tracks_active_one() {
        for i in 0..255 {
            let seed = [i; 32];

            let mut active_game = GameState::new(&seed, get_timestamp_now_ms());
            let mut passive_game = GameState::new(&seed, active_game.start_time);
            let mut _slices = vec![];

            loop {
                let action = TetAction::random();
                let res = active_game.try_action(action, get_timestamp_now_ms());
                if let Ok(new_active_game) = res {
                    active_game = new_active_game;
                } else {
                    continue;
                }
                if let GameReplaySegment::Update(ref update) = active_game.last_segment
                {
                    _slices.push(*update);
                }
                if active_game.game_over() {
                    break;
                }
            }

            for slice in _slices {
                tracing::info!("accept replay slice: {slice:?}");
                passive_game.accept_replay_slice(&slice).unwrap();
            }

            // assert_eq!(active_game, passive_game);
        }
    }
}

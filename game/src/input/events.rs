// tet actions:
// - HardDrop,
// - SoftDrop,
// - MoveLeft,
// - MoveRight,
// - Hold,
// - RotateLeft,
// - RotateRight,
//
// -- ESCAPE
// -- PAUSE
// -- MUTE_SOUND
// -- ZOOMIN_ZOOMOUT
//

use crate::tet::TetAction;

#[derive(Clone, Debug)]
pub struct GameInputEvent {
    pub key: GameInputEventKey,
    pub event: GameInputEventType,
    pub ts: chrono::DateTime<chrono::Utc>,
}

#[derive(
    Clone,
    Debug,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum GameInputEventType {
    KeyDown,
    KeyUp,
    // KeyPress,
}

#[derive(
    strum_macros::EnumString,
    strum_macros::EnumIter,
    Clone,
    Debug,
    Copy,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum GameInputEventKey {
    // game play
    HardDrop,
    SoftDrop,
    MoveLeft,
    MoveRight,
    Hold,
    RotateLeft,
    RotateRight,
    // menu
    MenuEscape,
    MenuPause,
    MenuMuteSound,
    MenuZoomIn,
    MenuZoomOut,
    // ???
    NoOp,
}

impl GameInputEventKey {
    pub fn to_game_action(&self) -> Option<TetAction> {
        match self {
            GameInputEventKey::HardDrop => Some(TetAction::HardDrop),
            GameInputEventKey::SoftDrop => Some(TetAction::UserSoftDrop),
            GameInputEventKey::MoveLeft => Some(TetAction::MoveLeft),
            GameInputEventKey::MoveRight => Some(TetAction::MoveRight),
            GameInputEventKey::Hold => Some(TetAction::Hold),
            GameInputEventKey::RotateLeft => Some(TetAction::RotateLeft),
            GameInputEventKey::RotateRight => Some(TetAction::RotateRight),
            _ => None,
        }
    }
}

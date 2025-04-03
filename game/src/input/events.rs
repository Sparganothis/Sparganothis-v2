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
    KeyPress,
}

#[derive(
    strum_macros::EnumString,
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

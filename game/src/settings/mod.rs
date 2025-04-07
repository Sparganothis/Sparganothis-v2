use std::time::Duration;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
    Default,
)]
pub struct GameSettings {
    pub input: GameInputSettings,
    pub game: GameModeSettings,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct GameInputSettings {
    pub autorepeat_delay_initial: Duration,
    pub autorepeat_delay_after: Duration,
}

impl Default for GameInputSettings {
    fn default() -> Self {
        Self {
            autorepeat_delay_initial: Duration::from_millis(167),
            autorepeat_delay_after: Duration::from_millis(33),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct GameModeSettings {
    pub auto_softdrop_interval: Duration,
}

impl Default for GameModeSettings {
    fn default() -> Self {
        Self {
            auto_softdrop_interval: Duration::from_millis(410),
        }
    }
}

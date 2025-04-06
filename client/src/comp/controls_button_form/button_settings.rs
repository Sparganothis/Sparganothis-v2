use std::collections::{BTreeMap, HashMap};

use dioxus::events::Code;
use game::input::events::GameInputEventKey;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct ButtonSettings {
    pub map:  HashMap<Code, GameInputEventKey>
}

impl Default for ButtonSettings {
    fn default() -> Self {
        Self { map: default_keymap() }
    }
}

fn default_keymap() -> HashMap<Code, GameInputEventKey> {
    let mut map: HashMap<Code, GameInputEventKey> = HashMap::new();

    map.insert(Code::KeyX, GameInputEventKey::RotateRight);
    map.insert(Code::ControlLeft, GameInputEventKey::RotateRight);
    map.insert(Code::ControlRight, GameInputEventKey::RotateRight);
    map.insert(Code::ArrowUp, GameInputEventKey::RotateRight);

    map.insert(Code::ArrowDown, GameInputEventKey::SoftDrop);
    map.insert(Code::Space, GameInputEventKey::HardDrop);
    map.insert(Code::Enter, GameInputEventKey::HardDrop);
    map.insert(Code::NumpadEnter, GameInputEventKey::HardDrop);
    map.insert(Code::Numpad0, GameInputEventKey::HardDrop);
    map.insert(Code::KeyZ, GameInputEventKey::RotateLeft);
    map.insert(Code::ArrowLeft, GameInputEventKey::MoveLeft);
    map.insert(Code::ArrowRight, GameInputEventKey::MoveRight);

    map.insert(Code::KeyC, GameInputEventKey::Hold);
    map.insert(Code::ShiftLeft, GameInputEventKey::Hold);
    map.insert(Code::ShiftRight, GameInputEventKey::Hold);

    map.insert(Code::Escape, GameInputEventKey::MenuEscape);
    map.insert(Code::KeyM, GameInputEventKey::MenuMuteSound);
    map.insert(Code::KeyP, GameInputEventKey::MenuPause);
    map.insert(Code::Minus, GameInputEventKey::MenuZoomIn);
    map.insert(Code::Equal, GameInputEventKey::MenuZoomOut);

    map
}
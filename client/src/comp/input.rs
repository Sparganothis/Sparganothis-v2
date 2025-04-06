use std::collections::HashMap;

use dioxus::prelude::*;

use game::input::events::{
    GameInputEvent, GameInputEventKey, GameInputEventType,
};

#[component]
pub fn GameInputCaptureParent(
    on_user_event: Callback<GameInputEvent, ()>,
    children: Element,
) -> Element {
    let on_key_cb = Callback::new(move |(key, event)| {
        let ts = protocol::datetime_now();
        let event = GameInputEvent { ts, key, event };
        on_user_event.call(event);
    });

    // https://github.com/DioxusLabs/dioxus/blob/bdf87aadc748c7531041b1f89166ebd64d7c8c26/examples/all_events.rs#L41
    rsx! {
        div {
            id: "event_input_capture_parent",
            tabindex: 0,
            style: "
                margin: 0px;
                padding: 0px; 
                border: 1px solid pink; 
                width: 100%; height: 100%;
                ",
            // onkeypress: move |_e| {
            //     info!("onkeypress: {:#?}", _e);
            //     if let Some(key) = keyboard_data_to_game_key(&_e.data()) {
            //         on_key_cb.call((key, GameInputEventType::KeyPress));
            //     }
            // },
            onkeyup: move |_e| {
                // info!("onkeyup: {:#?}", _e);
                if let Some(key) = keyboard_data_to_game_key(&_e.data()) {
                    _e.prevent_default();
                    _e.stop_propagation();

                    on_key_cb.call((key, GameInputEventType::KeyUp));
                }
            },
            onkeydown: move |_e| {
                // info!("onkeydown: {:#?}", _e);
                if let Some(key) = keyboard_data_to_game_key(&_e.data()) {
                    _e.prevent_default();
                    _e.stop_propagation();
                    
                    on_key_cb.call((key, GameInputEventType::KeyDown));
                }
            },
            onclick: move |_e| {
                // info!("cliek: {:#?}", _e);
            },
            onmounted: move |_e| {
                // info!("mounted! {:#?}", _e);
            },

            {children}
        }
    }
}

fn default_keymap() -> HashMap<Code, GameInputEventKey> {
    let mut map = HashMap::new();

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

fn keyboard_data_to_game_key(key: &KeyboardData) -> Option<GameInputEventKey> {
    if key.is_auto_repeating() {
        return None;
    }
    // https://github.com/Sparganothis/Sparganothis.github.io/blob/ba55b9523b4be3db6959d957965828d81a1ca83c/client/src/comp/hotkey_reader.rs#L94
    let map = default_keymap();
    map.get(&key.code()).copied()
}

use std::collections::HashMap;

use dioxus::prelude::*;

use game::input::events::{
    GameInputEvent, GameInputEventKey, GameInputEventType,
};

use crate::localstorage::use_button_settings;

use super::controls_button_form::ButtonSettings;

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
                let s = use_button_settings();

                if let Some(key) = keyboard_data_to_game_key(&_e.data(), &s) {
                    _e.prevent_default();
                    _e.stop_propagation();

                    on_key_cb.call((key, GameInputEventType::KeyUp));
                }
            },
            onkeydown: move |_e| {
                // info!("onkeydown: {:#?}", _e);
                let s = use_button_settings();
                if let Some(key) = keyboard_data_to_game_key(&_e.data(), &s) {
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



fn keyboard_data_to_game_key(key: &KeyboardData, btn_s: &ButtonSettings) -> Option<GameInputEventKey> {
    if key.is_auto_repeating() {
        return None;
    }
    // https://github.com/Sparganothis/Sparganothis.github.io/blob/ba55b9523b4be3db6959d957965828d81a1ca83c/client/src/comp/hotkey_reader.rs#L94
    btn_s.map.get(&key.code()).copied()
}

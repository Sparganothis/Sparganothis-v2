use std::time::Duration;

use dioxus::prelude::*;
use game::tet::GameState;

use crate::{
    comp::{
        singleplayer::{GameBoardInputAndDisplay, SingleplayerGameBoardBasic},
        slider::Slider,
    },
    localstorage::{set_game_settings, use_game_settings},
};

#[component]
pub fn SettingsForm() -> Element {
    rsx! {
        article {
            style: "
                display: flex;
                height: 100%;
                width: 100%;
                flex-direction: row;
            ",
            div {
                style: "
                    height: 100%;
                    width: 50%;
                    border: 1px solid green;
                    padding: 10px;
                    margin: 10px;
                ",
                GameSettingsForm {}
            }
            div {
                style: "
                    height: 100%;
                    width: 50%;
                    border: 1px solid magenta;
                    padding: 10px;
                    margin: 10px;
                ",
                GameSettingsInputPreview {}

            }
        }
    }
}

#[component]
pub fn GameSettingsForm() -> Element {
    rsx! {
        h4 {
            "Input Settings",
        }
        GameInputSettings{}
        h4 {
            "Difficulty Settings",
        }
        GameDifficultySettings{}
    }
}

#[component]
pub fn GameInputSettings() -> Element {
    rsx! {
        InitialRepeatDelaySlider{}
        AfterRepeatDelaySlider {}
    }
}

#[component]
fn InitialRepeatDelaySlider() -> Element {
    let old_settings = use_game_settings();
    let old_init =
        old_settings.input.autorepeat_delay_initial.as_millis() as u16;
    let label_initial = use_signal(|| "Initial Repeat Delay (ms)".to_string());
    let slidr_delay_init = use_signal(|| old_init);
    use_effect(move || {
        let init = slidr_delay_init.read().clone();
        let init = init.clamp(4, 666);
        if init != old_init {
            let mut new_settings = old_settings;
            new_settings.input.autorepeat_delay_initial =
                Duration::from_millis(init as u64);
            set_game_settings(new_settings);
        }
    });
    rsx! {
        h5 {
            "Initial Repeat Delay"
        }
        Slider {
            label: label_initial,
            m: slidr_delay_init,
            default_value: 155,
            min: 4,
            max: 666
        }
    }
}
#[component]
fn AfterRepeatDelaySlider() -> Element {
    let old_settings = use_game_settings();
    let old_init = old_settings.input.autorepeat_delay_after.as_millis() as u16;

    let label_initial = use_signal(|| "Repeat Delay (ms)".to_string());
    let slidr_delay_after = use_signal(|| old_init);
    use_effect(move || {
        let init = slidr_delay_after.read().clone();
        let init = init.clamp(4, 666);
        if init != old_init {
            let mut new_settings = old_settings;
            new_settings.input.autorepeat_delay_after =
                Duration::from_millis(init as u64);
            set_game_settings(new_settings);
        }
    });
    rsx! {
        h5 {
            "Repeat Delay"
        }
        Slider {
            label: label_initial,
            m: slidr_delay_after,
            default_value: 33,
            min: 4,
            max: 666
        }
    }
}

#[component]
pub fn GameDifficultySettings() -> Element {
    rsx! {
        GameDifficultyAutoSoftdropSlider {}
    }
}

#[component]
fn GameDifficultyAutoSoftdropSlider() -> Element {
    let old_settings = use_game_settings();
    let old_init = old_settings.game.auto_softdrop_interval.as_millis() as u16;

    let label_initial =
        use_signal(|| "Auto Soft Drop Interval (ms)".to_string());
    let slider_softdrop = use_signal(|| old_init);
    use_effect(move || {
        let init = slider_softdrop.read().clone();
        let init = init.clamp(4, 1111);
        if init != old_init {
            let mut new_settings = old_settings;
            new_settings.game.auto_softdrop_interval =
                Duration::from_millis(init as u64);
            set_game_settings(new_settings);
        }
    });
    rsx! {
        h5 {
            "Auto Soft Drop Interval"
        }
        Slider {
            label: label_initial,
            m: slider_softdrop,
            default_value: 250,
            min: 4,
            max: 1111
        }
    }
}

#[component]
pub fn GameSettingsInputPreview() -> Element {
    rsx! {
        SingleplayerGameBoardBasic {}
    }
}

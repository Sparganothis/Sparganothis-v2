use std::{fmt::Display, str::FromStr};

use dioxus::prelude::*;
use protocol::AcceptableType;

#[component]
pub fn Slider<T: Display + FromStr + AcceptableType>(
    label: ReadOnlySignal<String>,
    m: Signal<T>,
    default_value: T,
    min: T,
    max: T,
) -> Element {
    rsx! {
        label {
            "{label}: {m}"
            input {
                type: "range",
                min: "{min}",
                max: "{max}",
                value: "{m}",
                oninput: move |e| {
                    let value: String = e.value();
                    let value = value.parse::<T>();
                    if let Ok(value) = value {
                        m.set(value);
                    }
                }
            }
        }
    }
}

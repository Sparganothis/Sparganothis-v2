use crate::network::NetworkState;
use dioxus::prelude::*;
use protocol::ReceivedMessage;
use tracing::warn;

use super::chat_traits::ChatMessageType;

#[component]
pub fn ChatInput<T: ChatMessageType>(
    on_user_message: Callback<T::M, Option<ReceivedMessage<T>>>,
) -> Element {
    let mut message_input = use_signal(String::new);
    let is_connected = use_context::<NetworkState>().is_connected;

    let send_message = Callback::new(move |_: ()| {
        let mut _i = message_input.write();
        let message = _i.clone();
        let message = T::from_user_input(message);
        let m = on_user_message.call(message.clone());
        if let Some(_m) = m {
            _i.clear();
        } else {
            warn!("Failed to send message");
        }
    });
    let disabled = use_memo(move || {
        let m = message_input.read().clone();
        let is_connected = is_connected.read().clone();
        if m.trim().len() < 1 {
            return true;
        }
        if !is_connected {
            return true;
        };
        false
    });
    rsx! {
        article {
            role: "group",
            input {
                value: "{message_input.read()}",
                oninput: move |e| {
                    *message_input.write() = e.value();
                },
                onkeyup: move |e| {
                    if e.key() == Key::Enter {
                        if *disabled.read() {
                            e.prevent_default();
                            return;
                        }
                        send_message.call(());
                    }
                }
            }
            button { onclick: move |_| send_message.call(()), disabled: disabled,  "Send" }
        }
    }
}

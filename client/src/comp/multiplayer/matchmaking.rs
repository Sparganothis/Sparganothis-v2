use std::collections::{HashSet, VecDeque};

use dioxus::{html::mover, prelude::*};
use game::api::game_match::GameMatchType;
use protocol::{
    chat::{IChatController, IChatReceiver},
    global_chat::{
        GlobalChatMessageContent, MatchHandshakeType, MatchmakingMessage,
    },
    user_identity::NodeIdentity,
};
use tracing::info;

use crate::{comp::chat::chat_signals_hook::use_global_chat_controller_signal, network::NetworkState};

#[component]
pub fn MatchmakingWindow(
    user_match_type: ReadOnlySignal<GameMatchType>,
) -> Element {
    let is_connected = use_context::<NetworkState>().is_connected;
    if *is_connected.read() == false {
        return  rsx! {
            h1 {
                "Loading..."
            }
        }
    }
    info!("MatchmakingWindow");
    let chat = use_global_chat_controller_signal();

    let mut confirmed_game_w = use_signal(move || None);
    let confirmed_game = use_memo(move || confirmed_game_w.read().clone());
    let mut blacklist_w = use_signal(move || HashSet::new());
    let blacklist = use_memo(move || blacklist_w.read().clone());
    let on_lfg_message = Callback::new(move |x| {
        info!("recv LFG!");
        let user_match_type = user_match_type.read().clone();
        let (lfg_match_type, message_from) = x;
        if lfg_match_type != user_match_type {
            return;
        }
        if confirmed_game.read().is_some() {
            return;
        }
        let reply =
            GlobalChatMessageContent::handshake_request(user_match_type);
        chat.send_direct_user_message.call((message_from, reply));
        info!("send request");
    });

    let on_handshake_message = Callback::new(move |x| {
        let (match_type, from, hand_type) = x;
        info!("recv handshake {hand_type:?}!");
        
        let user_match_type = user_match_type.read().clone();
        if match_type != user_match_type {
            return;
        }
        let confirmed = confirmed_game.read().clone();
        match hand_type {
            MatchHandshakeType::HandshakeRequest => {
                info!("get request");
                if confirmed.is_some() {
                    // answer NO
                    let no = GlobalChatMessageContent::handshake_no(match_type);
                    chat.send_direct_user_message.call((from, no));
                    info!("send no");
                    return;
                }
                let yes = GlobalChatMessageContent::handshake_yes(match_type);
                chat.send_direct_user_message.call((from, yes));
                info!("send yes");
                confirmed_game_w.set(Some(from));
            }
            MatchHandshakeType::AnswerYes => {
                info!("get yes");
                if confirmed_game.read().is_none() {
                    confirmed_game_w.set(Some(from));
                }
            }
            MatchHandshakeType::AnswerNo => {
                info!("get no");
                blacklist_w.write().insert(from);
            }
        }
    });

    let mut past_msg: Signal<VecDeque<(MatchmakingMessage, NodeIdentity)>> =
        use_signal(move || VecDeque::new());

    let _chat_reader = use_resource(move || {
        let cc = chat.chat.read().clone();

        async move {
            let Some(cc) = cc else {
                return;
            };
            let recv = cc.receiver().await;
            while let Some(received_message) = recv.next_message().await {
                let content = received_message.message;
                match content {
                    GlobalChatMessageContent::MatchmakingMessage {
                        ref msg,
                    } => {
                        let mut past_msg = past_msg.write();
                        past_msg
                            .push_back((msg.clone(), received_message.from));
                        if past_msg.len() > 3 {
                            past_msg.pop_front();
                        }
                        match &msg {
                            MatchmakingMessage::LFG { match_type } => {
                                
                                on_lfg_message.call((
                                    match_type.clone(),
                                    received_message.from,
                                ));
                            }
                            MatchmakingMessage::Handshake {
                                match_type,
                                handshake_type,
                            } => {
                                on_handshake_message.call((
                                    match_type.clone(),
                                    received_message.from,
                                    handshake_type.clone(),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    rsx! {

        h1 {"old msg"}
        pre {
            "{past_msg:#?}"
        },

        h1 {"confirmed game? "}
        pre {
            "{confirmed_game:#?}"
        },

        h1 {"blacklist: "}
        pre {
            "{blacklist:#?}"
        },

        button {
            onclick: move |_| {
                info!("send LFG");
                chat.send_broadcast_user_message.call(GlobalChatMessageContent::MatchmakingMessage {
                    msg: MatchmakingMessage::LFG { match_type: game::api::game_match::GameMatchType::_1v1 }
                });
            },
            "Send msg. LFG 1v1"
        }
    }
}

use std::{
    collections::{HashSet, VecDeque},
    time::Duration,
};

use dioxus::{html::elements, prelude::*};
use futures_util::FutureExt;
use game::api::game_match::{GameMatch, GameMatchType};
use n0_future::StreamExt;
use protocol::{
    chat::{IChatController, IChatReceiver},
    game_matchmaker::find_game,
    global_chat::{
        GlobalChatMessageContent, MatchHandshakeType, MatchmakingMessage,
    },
    user_identity::NodeIdentity,
};
use tracing::{info, warn};

use crate::{
    app::GlobalUrlContext,
    comp::multiplayer::_1v1,
    network::{GlobalChatClientContext, NetworkState},
};

#[component]
pub fn MatchmakingWindow(
    user_match_type: ReadOnlySignal<GameMatchType>,
    on_opponent_confirm: Callback<GameMatch<NodeIdentity>>,
    on_matchmaking_failed: Callback<String>,
) -> Element {
    let chat = use_context::<GlobalChatClientContext>().chat;
    let mut clicked = use_signal(|| false);

    let mut timer_w = use_signal(move || 0);
    let timer = use_memo(move || timer_w.read().clone());
    let attempt_timeout_secs = 10;
    let attempts = 5;
    let coro = use_coroutine(move |mut _r| async move {
        while let Some(_x) = _r.next().await {
            let Some(global_chat) = chat.chat.peek().clone() else {
                warn!("no chat!");
                continue;
            };

            info!("\n\n\n >>>>> > START MATCHMAKING \n\n");

            let timeout = std::time::Duration::from_secs(attempt_timeout_secs);
            let mut game_fut =
                find_game(user_match_type.peek().clone(), global_chat, timeout, attempts).fuse().boxed();
            timer_w.set(0);
            let game = loop {
                tokio::select! {
                    _ = n0_future::time::sleep(Duration::from_secs(1)).fuse() => {
                        let old = timer.peek().clone() + 1;
                        if old > (attempt_timeout_secs + 1) * attempts as u64 {
                            break Err("coro timeout".to_string());
                        }
                        timer_w.set(old);

                        continue;
                    }
                    game = &mut game_fut => {
                        break game.map_err(|e| e.to_string());
                    }
                }
            };
            match game {
                Ok(from) => {
                    on_opponent_confirm.call(from);
                }
                Err(e) => {
                    let txt = format!("err: {e}");
                    on_matchmaking_failed.call(txt.clone());
                }
            }
            
            info!("\n\n\n >>>>> > END MATCHMAKING \n\n");
        }
    });

    let mm = use_context::<NetworkState>().is_connected;

    rsx! {
        div {
            style: "
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                height: 4.5rem;
                width: 15rem;
            ",
            if *mm.read() {
                if !*clicked.read() {
                    button {
                        onclick: move |_| {
                            coro.send(());
                            clicked.set(true);
                        },
                        "Look for game!"
                    }
                } else {
                    h3 {
                        "Looking {timer}/{attempt_timeout_secs}..."
                    }
                }
            } else {
                h3 {
                    "Connecting..."
                }
            }
        }

    }
}
// #[component]
// pub fn MatchmakingWindow(
//     user_match_type: ReadOnlySignal<GameMatchType>,
//     on_opponent_confirm: Callback<NodeIdentity>,
//     on_reset: Callback<()>,
// ) -> Element {
//     let is_connected = use_context::<NetworkState>().is_connected;
//     if *is_connected.read() == false {
//         return  rsx! {
//             h1 {
//                 "Loading..."
//             }
//         }
//     }
//     info!("MatchmakingWindow");
//     let chat = use_global_chat_controller_signal();
//     let mut search_active_w = use_signal(move || false);
//     let search_active = use_memo(move || search_active_w.read().clone());

//     let mut confirmed_game_w = use_signal(move || None);
//     let confirmed_game = use_memo(move || confirmed_game_w.read().clone());
//     let mut blacklist_w = use_signal(move || HashSet::new());
//     let blacklist = use_memo(move || blacklist_w.read().clone());

//     use_effect(move || {
//         let item = confirmed_game.read().clone();
//         if let Some(item) = item {
//             confirmed_game_w.set(None);
//             blacklist_w.set(HashSet::new());
//             on_opponent_confirm.call(item);
//             search_active_w.set(false);
//         }
//     });

//     let on_lfg_message = Callback::new(move |x| {
//         if !*search_active.read() {
//             return;
//         }
//         info!("recv LFG!");
//         let user_match_type = user_match_type.read().clone();
//         let (lfg_match_type, message_from) = x;
//         if lfg_match_type != user_match_type {
//             return;
//         }
//         if confirmed_game.read().is_some() {
//             return;
//         }
//         let reply =
//             GlobalChatMessageContent::handshake_request(user_match_type);
//         chat.send_direct_user_message.call((message_from, reply));
//         info!("send request");
//     });

//     let on_handshake_message = Callback::new(move |x| {
//         if !*search_active.read() {
//             return;
//         }

//         let (match_type, from, hand_type) = x;
//         info!("recv handshake {hand_type:?}!");

//         let user_match_type = user_match_type.read().clone();
//         if match_type != user_match_type {
//             return;
//         }
//         let confirmed = confirmed_game.read().clone();
//         match hand_type {
//             MatchHandshakeType::HandshakeRequest => {
//                 info!("get request");
//                 if confirmed.is_some() {
//                     // answer NO
//                     let no = GlobalChatMessageContent::handshake_no(match_type);
//                     chat.send_direct_user_message.call((from, no));
//                     info!("send no");
//                     return;
//                 }
//                 let yes = GlobalChatMessageContent::handshake_yes(match_type);
//                 chat.send_direct_user_message.call((from, yes));
//                 info!("send yes");
//                 confirmed_game_w.set(Some(from));
//             }
//             MatchHandshakeType::AnswerYes => {
//                 info!("get yes");
//                 if confirmed_game.read().is_none() {
//                     confirmed_game_w.set(Some(from));
//                 }
//             }
//             MatchHandshakeType::AnswerNo => {
//                 info!("get no");
//                 blacklist_w.write().insert(from);
//             }
//         }
//     });

//     let mut past_msg: Signal<VecDeque<(MatchmakingMessage, NodeIdentity)>> =
//         use_signal(move || VecDeque::new());

//     let _chat_reader = use_resource(move || {
//         let cc = chat.chat.read().clone();
//         let active = search_active.read().clone();

//         async move {
//             if !active {
//                 return;
//             }
//             info!("CHAT_READER LOOP START");
//             let Some(cc) = cc else {
//                 return;
//             };
//             let recv = cc.receiver().await;
//             while let Some(received_message) = recv.next_message().await {
//                 let content = received_message.message;
//                 match content {
//                     GlobalChatMessageContent::MatchmakingMessage {
//                         ref msg,
//                     } => {
//                         let mut past_msg = past_msg.write();
//                         past_msg
//                             .push_back((msg.clone(), received_message.from));
//                         if past_msg.len() > 3 {
//                             past_msg.pop_front();
//                         }
//                         if !* search_active.read() {
//                             continue;
//                         }
//                         match &msg {
//                             MatchmakingMessage::LFG { match_type } => {
//                                 on_lfg_message.call((
//                                     match_type.clone(),
//                                     received_message.from,
//                                 ));
//                             }
//                             MatchmakingMessage::Handshake {
//                                 match_type,
//                                 handshake_type,
//                             } => {
//                                 on_handshake_message.call((
//                                     match_type.clone(),
//                                     received_message.from,
//                                     handshake_type.clone(),
//                                 ));
//                             }
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     });

//     let btn_disabled = use_memo( move || {
//         let have_chat = chat.chat.read().clone().is_some();
//         !have_chat
//     });

//     let _execute = use_coroutine(move |mut _r: UnboundedReceiver<()>| async move {
//         while let Some(_x) = _r.next().await {
//             info!("EXECUTE MATCHMAKER");
//             confirmed_game_w.set(None);
//             blacklist_w.set(HashSet::new());
//             search_active_w.set(true);
//             n0_future::time::sleep(Duration::from_millis(1)).await;
//             chat.send_broadcast_user_message.call(
//                     GlobalChatMessageContent::MatchmakingMessage {
//                 msg: MatchmakingMessage::LFG {
//                      match_type: user_match_type.read().clone() }
//             });
//         }
//     });

//     // when page url changes, reset the state
//     use_effect (move || {
//         let _url = use_context::<GlobalUrlContext>().url.read().clone();
//         confirmed_game_w.set(None);
//         blacklist_w.set(HashSet::new());
//     });

//     // todo: on confirm send message from button

//     rsx! {
//         button {
//             onclick: move |_| {
//                 _execute.send(());
//             },
//             disabled: *btn_disabled.read(),
//             "Send msg. LFG 1v1"
//         }
//         button {
//             class: "secondary",
//             onclick: move |_| {
//                 info!("RESET MATCHMAKER");
//                 confirmed_game_w.set(None);
//                 blacklist_w.set(HashSet::new());
//                 past_msg.set(VecDeque::new());
//                 search_active_w.set(false);

//                 on_reset.call(());
//             },
//             "Reset Matchmaker"
//         }

//         h1 {"confirmed game? "}
//         pre {
//             "{confirmed_game:#?}"
//         },

//         h1 {"blacklist: "}
//         pre {
//             "{blacklist:#?}"
//         },

//         h1 {"old msg"}
//         pre {
//             "{past_msg:#?}"
//         },

//     }
// }

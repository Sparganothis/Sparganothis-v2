use std::collections::BTreeSet;

use dioxus::prelude::*;
use game::tet::GameState;
use crate::{comp::{bot_player::BotPlayer, game_display::*}, network::NetworkState, pages::{GameMessage, GameMessageSpam}, route::Route};
use protocol::{chat::{IChatController, IChatSender}, chat_ticket::ChatTicket};
use tracing::{info, warn};
#[component]
pub fn Singleplayer() -> Element {
    let game_state = use_signal(GameState::empty);
    let mut url_sig = use_signal(|| "".to_string());
    let mut cc_signal = use_signal(|| None);
    let mm = use_context::<NetworkState>().global_mm;
    let _link_sender = use_resource(move || {
        let mm = mm.read().clone();
        async move {
            let Some( mm) = mm else {
                warn!("No global matchmaker");
                return;
            };
            let Some(cc) = mm.global_chat_controller().await else {
                warn!("No global chat controller");
                return;
            };
            info!("mm ok");
            let url = Route::SpectateGamePage { node_id: * mm.own_node_identity().node_id() }.to_string();
            let url = format!("http://localhost:8080/Sparganothis-v2{}", url);
            url_sig.set(url.clone());
            warn!("Sending url: {url}");
            let _r  = cc.sender().broadcast_message(url).await;
            if let Err(e) = _r {
                warn!("Failed to send message to global chat: {e}");
            }
            warn!("SEND OK!!!");
        }
    });
    let _move_spammer = use_resource(move || {
        let mm = mm.read().clone();
        async move {
            let Some( mm) = mm else {
                warn!("No global matchmaker");
                return;
            };
            let Some(nn) = mm.own_node().await else {
                warn!("No own node");
                return;
            };
            let chat_ticket = ChatTicket::new_str_bs("play", BTreeSet::from([]));
            let Ok(chat) = nn.join_chat::<GameMessageSpam>(&chat_ticket).await else {
                warn!("Failed to join chat");
                return;
            };
            cc_signal.set(Some(chat));
        }
    });
    let _on_game_change_send_to_chat = use_resource(move || {
        let game = game_state.read().clone();
        let chat = cc_signal.read().clone();
        async move {
            let Some(chat) = chat else {
                warn!("No chat");
                return;
            };
            let _r = chat.sender().broadcast_message(GameMessage::GameState(game)).await;
            if let Err(e) = _r {
                warn!("Failed to send message to game chat: {e}");
            }
        }
    });

    rsx! {
        article { style: "height: 80dvh; display: flex;",
            // style: "display: flex;",

            a {
                style: "width: 100px; height: 100px; border: 1px solid black;",
                href: url_sig.read().clone(),
                "Spectate {url_sig}"
            }
            BotPlayer { game_state }

            GameDisplay { game_state }
        }
    }
}

use std::collections::BTreeSet;

use crate::{
    comp::{
        bot_player::BotPlayer, chat::chat_signals_hook::use_chat_signals,
        game_display::*,
    },
    network::GlobalChatClientContext,
    pages::{GameMessage, GameMessageSpam},
    route::Route,
};
use dioxus::prelude::*;
use game::tet::GameState;
use protocol::{
    chat::IChatController, chat_ticket::ChatTicket,
    global_matchmaker::GlobalMatchmaker,
};
use tracing::{info, warn};
#[component]
pub fn IAmARobotSingleplayer() -> Element {
    let game_state = use_signal(GameState::empty);
    let mut url_sig = use_signal(|| "".to_string());
    let global_chat = use_context::<GlobalChatClientContext>().chat;
    let _link_sender = use_resource(move || {
        let cc = global_chat.chat.read().clone();
        async move {
            let Some(cc) = cc else {
                warn!("No global chat controller");
                return;
            };
            info!("mm ok");
            let url = Route::SpectateGamePage {
                node_id: *cc.node_identity().node_id(),
            }
            .to_string();
            let url = format!("http://localhost:8080/Sparganothis-v2{}", url);
            url_sig.set(url.clone());
            warn!("Sending url: {url}");
            global_chat.send_user_message.call(url.into());
        }
    });
    let game_chat = use_chat_signals(Callback::new(
        move |mm: GlobalMatchmaker| async move {
            let chat_ticket =
                ChatTicket::new_str_bs("play", BTreeSet::from([]));
            let node = mm.own_node().await?;
            let chat =
                node.join_chat::<GameMessageSpam>(&chat_ticket).await.ok()?;
            Some(chat)
        },
    ));
    let _on_game_change_send_to_chat = use_resource(move || {
        let game = game_state.read().clone();
        async move {
            game_chat
                .send_user_message
                .call(GameMessage::GameState(game));
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

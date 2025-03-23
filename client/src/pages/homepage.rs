use std::{collections::{BTreeSet, VecDeque}, time::Duration};

use crate::{comp::{game_display::*, slider::Slider}, network::NetworkState, pages::{GameMessage, GameMessageSpam}, route::Route};
use dioxus::prelude::*;
use dioxus_sdk::utils::timing::use_interval;
use game::{
    bot::{wordpress_blog_bot::WordpressBlogBot, TetBot},
    tet::{GameState, TetAction},
};
use protocol::chat::ChatTicket;
use tracing::warn;
/// Home page
#[component]
pub fn Home() -> Element {
    let mut game_state = use_signal(GameState::empty);
    let mut url_sig = use_signal(|| "".to_string());
    let mut pending_actions = use_signal(VecDeque::<TetAction>::new);
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
            let url = Route::SpectateGamePage { node_id: * mm.own_node_identity().node_id() }.to_string();
            let url = format!("http://localhost:8080/Sparganothis-v2{}", url);
            url_sig.set(url.clone());
            warn!("Sending url: {url}");
            let _r  = cc.sender().send(url).await;
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
            let _r = chat.sender().send(GameMessage::GameState(game)).await;
            if let Err(e) = _r {
                warn!("Failed to send message to game chat: {e}");
            }
        }
    });
    let speed = use_signal(|| 3.0);
    let _ = use_resource(move || {
        
        let speed = speed.read().clone();
        async move { loop {
        let mut g = game_state.write();
        let mut p = pending_actions.write();

        n0_future::time::sleep(Duration::from_secs_f32(speed)).await;

        if g.game_over() {
            *g = GameState::empty();
            return;
        }
        if p.is_empty() {
            if let Ok(r) = WordpressBlogBot.choose_move(&g) {
                *p = VecDeque::from_iter(r.into_iter());
            }
        }

        if let Some(a) = p.pop_front() {
            if let Ok(new_state) = g.try_action(a, 0) {
                *g = new_state;
            }
        }
    }}});

    rsx! {
        article { style: "height: 80dvh; display: flex;",
            // style: "display: flex;",

            a {
                style: "width: 100px; height: 100px; border: 1px solid black;",
                href: url_sig.read().clone(),
                "Spectate {url_sig}"
            }
            Slider<f32> {
                label: "Speed".into(),
                m: speed,
                default_value: 3.0,
                min: 0.1,
                max: 3.0,
            }
            GameDisplay { game_state }
        }
    }
}

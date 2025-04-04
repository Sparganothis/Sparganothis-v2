use anyhow::Context;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::{pin_mut, FutureExt};
use n0_future::task::AbortOnDropHandle;
use protocol::chat::{IChatController, IChatReceiver, IChatSender};
use protocol::global_matchmaker::{GlobalChatPresence, GlobalMatchmaker};
use protocol::user_identity::UserIdentitySecrets;
use ratatui::DefaultTerminal;

use n0_future::stream::StreamExt;
use n0_future::task::spawn;

use crate::terminal_ui::app_state::{ChatWindowData, SpecificWindowData, WindowData};
use crate::terminal_ui::draw::draw_main;

use super::app_state::AppState;

pub async fn terminal_loop(
    terminal: &mut DefaultTerminal,
) -> anyhow::Result<()> {
    let app_state = AppState::new();
    let app_state_ = app_state.clone();

    let event_stream = EventStream::new();
    let (event_tx, event_rx) = tokio::sync::mpsc::channel::<Event>(1);
    let app_driver_task = AbortOnDropHandle::new(spawn(async move {
        app_driver(app_state_, event_rx).await
    }));
    let mut app_driver_task = app_driver_task.fuse();

    pin_mut!(event_stream);
    while !app_state.should_exit().await {
        tokio::select! {
            event = event_stream.next().fuse() => {
                let Some(Ok(event)) = event else {
                    anyhow::bail!("Event stream closed");
                };
                if is_exit_event(&event) {
                    return Ok(());
                }
                event_tx.send(event).await?;
            }
            _ = app_state.notified().fuse() => {
                let data = app_state.get_state().await;
                terminal.draw(move |frame| {
                    draw_main(frame, &data);
                })?;
            }
            _ = n0_future::time::sleep(std::time::Duration::from_secs(1)).fuse() => {
                let data = app_state.get_state().await;
                terminal.draw(move |frame| {
                    draw_main(frame, &data);
                })?;
            }
            _ = &mut app_driver_task => {
                anyhow::bail!("App Driver Ended")
            }
        }
    }
    anyhow::bail!("Main Loop Ended")
}

fn is_exit_event(event: &Event) -> bool {
    match event {
        Event::Key(key) => match key.code {
            KeyCode::Char('c') => key.modifiers.contains(KeyModifiers::CONTROL),
            KeyCode::Esc => true,
            _ => false,
        },
        _ => false,
    }
}
async fn app_driver(
    app_state: AppState,
    event_rx: Receiver<Event>,
) -> anyhow::Result<()> {
    app_state
        .set_loading_message("Generating identity...")
        .await;
    let id = UserIdentitySecrets::generate();
    app_state
        .set_loading_message("Connecting to server...")
        .await;

    let global_mm = GlobalMatchmaker::new(Arc::new(id)).await?;

    app_state.set_loading_message("Connecting to chat...").await;

    let _r = chat_driver(app_state, global_mm.clone(), event_rx).await;
    global_mm.shutdown().await?;
    _r
}

async fn chat_driver(
    app_state: AppState,
    global_mm: GlobalMatchmaker,
    event_rx: Receiver<Event>,
) -> anyhow::Result<()> {
    let controller = global_mm.global_chat_controller().await.context("F")?;
    let sender = controller.sender();

    sender
        .set_presence(&GlobalChatPresence {
            url: "".to_string(),
            platform: "Terminal UI".to_string(),
        })
        .await;

    app_state.set_loading_message("Waiting for chat...").await;
    controller.wait_joined().await?;

    app_state
        .set_state(SpecificWindowData::Chat(ChatWindowData {
            own_identity: controller.node_identity(),
            presence: vec![],
            msg_history: vec![],
        }))
        .await;

    let presence = controller.chat_presence();
    let recv = controller.receiver().await;

    futures::pin_mut!(event_rx);
    loop {
        tokio::select! {
            event = event_rx.recv().fuse() => {
                let Some(_event) = event else {
                    anyhow::bail!("Event stream closed");
                };
            }
            _ = presence.notified().fuse() => {
                let presence_list = presence.get_presence_list().await;
                app_state.set_presence_list(presence_list).await;
            }
            msg = recv.next_message().fuse() => {
                let Some(msg) = msg else {
                    anyhow::bail!("Message stream closed");
                };
                app_state.append_msg_history(msg).await;
            }
        }
    }
}

use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures::{pin_mut, FutureExt};
use ratatui::DefaultTerminal;

use n0_future::stream::StreamExt;

use crate::terminal_ui::router::{make_router, Page, PageRoute};

use super::router::Router;

pub async fn terminal_main_loop(
    terminal: &mut DefaultTerminal,
) -> anyhow::Result<()> {
    let router = make_router();

    let r = terminal_inner_loop(terminal, router.clone()).await;
    router.shutdown().await;
    drop(router);
    r
}

async fn terminal_inner_loop(
    terminal: &mut DefaultTerminal,
    router: Router,
) -> anyhow::Result<()> {
    let event_stream = EventStream::new();
    let (event_tx, event_rx) = tokio::sync::mpsc::channel::<Event>(16);
    let app_driver_task = router.handle_event_stream(event_rx);
    let mut app_driver_task = app_driver_task.fuse();

    router.open_page(PageRoute::Singleplayer).await?;

    pin_mut!(event_stream);
    loop {
        let page = router.get_page().await;
        if let Some(page) = page {
            let page2 = page.get_drawable().await;
            terminal.draw(move |frame| {
                page2.draw(frame);
            })?;
        }
        tokio::select! {
            event = event_stream.next().fuse() => {
                let Some(Ok(event)) = event else {
                    anyhow::bail!("Event stream closed");
                };
                if is_exit_event(&event) {
                    return Ok(());
                }
                if router.handle_navigation_event(&event).await? {
                    continue;
                }
                event_tx.send(event).await?;
            }
            _ = router.notified().fuse() => {
                // nothing special
            }
            _ = n0_future::time::sleep(std::time::Duration::from_secs(10)).fuse() => {
                // nothing special
            }
            _ = &mut app_driver_task => {
                anyhow::bail!("App Driver Ended")
            }
        }
    }
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

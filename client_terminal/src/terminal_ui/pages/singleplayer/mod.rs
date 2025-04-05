use std::sync::Arc;

use crate::terminal_ui::router::{Drawable, DynamicPage, Page, PageFactory};
use async_trait::async_trait;
use crossterm::event::{Event, KeyCode};
use n0_future::task::AbortOnDropHandle;
use ratatui::widgets::Paragraph;
use tokio::sync::{Mutex, Notify};

#[derive(Debug)]
pub struct SingleplayerPageFactory;

impl PageFactory for SingleplayerPageFactory {
    fn create_page(&self, notify: Arc<Notify>) -> DynamicPage {
        let page = SingleplayerPage {
            _notify: notify,
            data: Arc::new(Mutex::new(SingleplayerPageState { x: 0 })),
        };
        let _page = page.clone();
        let task = async move {
            loop {
                n0_future::time::sleep(std::time::Duration::from_secs(1)).await;
                _page.increment().await;
            }
        };
        let task = AbortOnDropHandle::new(n0_future::task::spawn(task));
        DynamicPage::new(page, task)
    }
}

#[derive(Debug, Clone)]
pub struct SingleplayerPage {
    _notify: Arc<Notify>,
    data: Arc<Mutex<SingleplayerPageState>>,
}

#[derive(Debug, Clone)]
pub struct SingleplayerPageState {
    x: i32,
}

#[async_trait]
impl Page for SingleplayerPage {
    async fn get_drawable(&self) -> Box<dyn Drawable> {
        Box::new(self.data.lock().await.clone())
    }
    async fn shutdown(&self) {}
    async fn handle_event(&self, _event: Event) {
        let Event::Key(key) = _event else {
            return;
        };
        if key.kind != crossterm::event::KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Right => self.increment().await,
            KeyCode::Left => self.decrement().await,
            _ => {}
        }
    }
}

impl SingleplayerPage {
    async fn increment(&self) {
        {
            let mut data = self.data.lock().await;
            data.x += 1;
        }
        self._notify.notify_waiters();
    }
    async fn decrement(&self) {
        {
            let mut data = self.data.lock().await;
            data.x -= 1;
        }
        self._notify.notify_waiters();
    }
}

impl Drawable for SingleplayerPageState {
    fn draw(&self, frame: &mut ratatui::Frame) {
        let string = format!("SingleplayerPage: {}", self.x);
        frame.render_widget(Paragraph::new(string), frame.area());
    }
}

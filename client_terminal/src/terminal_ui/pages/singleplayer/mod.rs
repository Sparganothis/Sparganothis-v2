use std::sync::Arc;

use crate::terminal_ui::router::{Drawable, DynamicPage, Page, PageFactory};
use async_trait::async_trait;
use crossterm::event::{Event, KeyCode};
use game::tet::GameState;
use n0_future::task::AbortOnDropHandle;
use ratatui::widgets::Paragraph;
use tokio::sync::{Mutex, Notify};

#[derive(Debug)]
pub struct SingleplayerPageFactory;

impl PageFactory for SingleplayerPageFactory {
    fn create_page(&self, notify: Arc<Notify>) -> DynamicPage {
        let page = SingleplayerPage {
            _notify: notify,
            data: Arc::new(Mutex::new(SingleplayerPageState  { game_state: GameState::empty() })),
        };
        let _page = page.clone();
        let task = async move {
            loop {
                n0_future::time::sleep(std::time::Duration::from_secs(1)).await;
                
                {
                    let mut data = _page.data.lock().await;
                    if let Ok(next) = data.game_state.try_action(game::tet::TetAction::SoftDrop, 0) {
                        data.game_state = next;
                    } else {
                        data.game_state = GameState::empty();
                    }
                }
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
    game_state: GameState,
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
            _ => {}
        }
    }
}

impl SingleplayerPage {
}

impl Drawable for SingleplayerPageState {
    fn draw(&self, frame: &mut ratatui::Frame) {
        let string = format!("SingleplayerPage: {}", self.game_state.get_debug_matrix_txt());
        frame.render_widget(Paragraph::new(string), frame.area());
    }
}

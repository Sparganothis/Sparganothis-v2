use std::sync::Arc;

use crate::terminal_ui::router::{Drawable, DynamicPage, Page, PageFactory};
use async_trait::async_trait;
use crossterm::event::{Event, KeyCode, KeyEventKind, ModifierKeyCode};
use futures::StreamExt;
use game::{
    input::{
        callback_manager::CallbackManager,
        events::{GameInputEvent, GameInputEventKey, GameInputEventType},
    },
    settings::GameSettings,
    tet::GameState,
    timestamp::get_timestamp_now_ms,
};
use n0_future::task::AbortOnDropHandle;
use protocol::datetime_now;
use ratatui::widgets::Paragraph;
use tokio::sync::{Notify, RwLock};

#[derive(Debug)]
pub struct SingleplayerPageFactory;

impl PageFactory for SingleplayerPageFactory {
    fn create_page(&self, notify: Arc<Notify>) -> DynamicPage {
        let (event_tx, event_rx) = game::futures_channel::mpsc::unbounded::<(
            GameState,
            GameInputEvent,
        )>();
        let page = SingleplayerPage {
            _notify: notify,
            data: Arc::new(RwLock::new(SingleplayerPageState {
                game_state: GameState::empty(),
            })),
            event_tx,
        };
        let _page = page.clone();
        let task = async move {
            let callback_manager = CallbackManager::new2();
            let s = GameSettings::default();
            let s = Arc::new(RwLock::new(s));
            let _s = callback_manager.main_loop(event_rx, s.clone()).await;
            futures::pin_mut!(_s);
            while let Some(event) = _s.next().await {
                {
                    let mut data = _page.data.write().await;
                    if let Ok(next) = data
                        .game_state
                        .try_action(event, get_timestamp_now_ms())
                    {
                        data.game_state = next;
                    } else if data.game_state.game_over() {
                        data.game_state = GameState::empty();
                    }
                }
                _page._notify.notify_waiters();
                _page._notify.notify_one();
            }
        };
        let task = AbortOnDropHandle::new(n0_future::task::spawn(task));
        DynamicPage::new(page, task)
    }
}

#[derive(Debug, Clone)]
pub struct SingleplayerPage {
    _notify: Arc<Notify>,
    data: Arc<RwLock<SingleplayerPageState>>,
    event_tx: game::futures_channel::mpsc::UnboundedSender<(
        GameState,
        GameInputEvent,
    )>,
}

#[derive(Debug, Clone)]
pub struct SingleplayerPageState {
    game_state: GameState,
}

impl SingleplayerPage {
    async fn get_gamestate(&self) -> GameState {
        self.data.read().await.game_state.clone()
    }
}

#[async_trait]
impl Page for SingleplayerPage {
    async fn get_drawable(&self) -> Box<dyn Drawable> {
        Box::new(self.data.read().await.clone())
    }
    async fn shutdown(&self) {}
    async fn handle_event(&self, _event: Event) {
        let Event::Key(key) = _event else {
            return;
        };
        let key2 = match key.code {
            KeyCode::Char('z') => Some(GameInputEventKey::RotateLeft),

            KeyCode::Char('x') => Some(GameInputEventKey::RotateRight),
            KeyCode::Up => Some(GameInputEventKey::RotateRight),
            KeyCode::Modifier(ModifierKeyCode::LeftControl) => {
                Some(GameInputEventKey::RotateRight)
            }
            KeyCode::Modifier(ModifierKeyCode::RightControl) => {
                Some(GameInputEventKey::RotateRight)
            }

            KeyCode::Left => Some(GameInputEventKey::MoveLeft),
            KeyCode::Right => Some(GameInputEventKey::MoveRight),

            KeyCode::Down => Some(GameInputEventKey::SoftDrop),

            KeyCode::Char('c') => Some(GameInputEventKey::Hold),
            KeyCode::Modifier(ModifierKeyCode::LeftShift) => {
                Some(GameInputEventKey::Hold)
            }
            KeyCode::Modifier(ModifierKeyCode::RightShift) => {
                Some(GameInputEventKey::Hold)
            }

            KeyCode::Char(' ') => Some(GameInputEventKey::HardDrop),
            KeyCode::Enter => Some(GameInputEventKey::HardDrop),
            KeyCode::Char('0') => Some(GameInputEventKey::HardDrop),

            KeyCode::Char('q') => Some(GameInputEventKey::MenuEscape),
            KeyCode::Char('p') => Some(GameInputEventKey::MenuPause),
            KeyCode::Char('m') => Some(GameInputEventKey::MenuMuteSound),
            KeyCode::Char('+') => Some(GameInputEventKey::MenuZoomIn),
            KeyCode::Char('-') => Some(GameInputEventKey::MenuZoomOut),

            _ => None,
        };
        let event2 = match key.kind {
            KeyEventKind::Press => Some(GameInputEventType::KeyDown),
            KeyEventKind::Release => Some(GameInputEventType::KeyUp),
            _ => None,
        };
        if let Some(key) = key2 {
            if let Some(event) = event2 {
                let event = GameInputEvent {
                    ts: datetime_now(),
                    key,
                    event,
                };
                self.event_tx
                    .unbounded_send((self.get_gamestate().await, event))
                    .unwrap();
            }
        }
    }
}

impl SingleplayerPage {}

impl Drawable for SingleplayerPageState {
    fn draw(&self, frame: &mut ratatui::Frame) {
        let string = format!(
            "SingleplayerPage: {}",
            self.game_state.get_debug_matrix_txt()
        );
        frame.render_widget(Paragraph::new(string), frame.area());
    }
}

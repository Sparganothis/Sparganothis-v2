mod draw;

use std::sync::Arc;

use crate::terminal_ui::router::{Drawable, DynamicPage, Page, PageFactory};
use anyhow::Context;
use async_trait::async_trait;
use crossterm::event::Event;
use futures::FutureExt;
use n0_future::task::AbortOnDropHandle;
use protocol::{
    chat::{ChatSender, IChatController, IChatReceiver, IChatSender},
    chat_presence::PresenceList,
    global_chat::{GlobalChatPresence, GlobalChatRoomType},
    global_matchmaker::GlobalMatchmaker,
    user_identity::{NodeIdentity, UserIdentitySecrets},
    ReceivedMessage,
};
use tokio::sync::{Mutex, Notify};

#[derive(Debug)]
pub struct ChatPageFactory;

impl PageFactory for ChatPageFactory {
    fn create_page(&self, notify: Arc<Notify>) -> DynamicPage {
        let page = ChatPage {
            _notify: notify,
            data: Arc::new(Mutex::new(ChatPageState::new_loading(
                "Loading...".to_string(),
            ))),
            sender: Arc::new(Mutex::new(None)),
            mm: Arc::new(Mutex::new(None)),
        };
        let task = task_page_driver(page.clone());
        let task = async move {
            let _r = task.await;
            ()
        };
        let task = AbortOnDropHandle::new(n0_future::task::spawn(task));
        DynamicPage::new(page, task)
    }
}

#[derive(Clone)]
pub struct ChatPage {
    _notify: Arc<Notify>,
    data: Arc<Mutex<ChatPageState>>,
    sender: Arc<Mutex<Option<ChatSender<GlobalChatRoomType>>>>,
    mm: Arc<Mutex<Option<GlobalMatchmaker>>>,
}

#[derive(Debug, Clone)]
pub enum ChatPageState {
    ChatLoaded {
        own_identity: NodeIdentity,
        presence: PresenceList<GlobalChatRoomType>,
        msg_history: Vec<ReceivedMessage<GlobalChatRoomType>>,
        input_buffer: String,
        scroll_position: usize,
    },
    ChatLoading {
        message: String,
    },
}

#[async_trait]
impl Page for ChatPage {
    async fn get_drawable(&self) -> Box<dyn Drawable> {
        Box::new(self.data.lock().await.clone())
    }
    async fn shutdown(&self) {
        let i = self.mm.lock().await;
        if let Some(mm) = i.as_ref() {
            let _ = mm.shutdown().await;
        }
    }
    async fn handle_event(&self, event: Event) {
        match event {
            Event::Key(key_event) => {
                if key_event.kind != crossterm::event::KeyEventKind::Press {
                    return;
                }
                match key_event.code {
                    crossterm::event::KeyCode::Char(c) => {
                        self.handle_key(c).await;
                    }
                    crossterm::event::KeyCode::Backspace => {
                        self.handle_backspace().await;
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.send_message().await;
                    }
                    crossterm::event::KeyCode::PageUp => {
                        self.scroll_messages(-1).await;
                    }
                    crossterm::event::KeyCode::PageDown => {
                        self.scroll_messages(1).await;
                    }
                    _ => {}
                }
            }
            Event::Mouse(mouse_event) => {
                if let crossterm::event::MouseEventKind::ScrollUp =
                    mouse_event.kind
                {
                    self.scroll_messages(-1).await;
                } else if let crossterm::event::MouseEventKind::ScrollDown =
                    mouse_event.kind
                {
                    self.scroll_messages(1).await;
                }
            }
            _ => {}
        }
    }
}

impl Drawable for ChatPageState {
    fn draw(&self, frame: &mut ratatui::Frame) {
        draw::draw_chat(frame, self);
    }
}

impl ChatPage {
    pub async fn send_message(&self) {
        if let Some(sender) = &self.sender.lock().await.as_ref() {
            let mut state = self.data.lock().await;
            if let ChatPageState::ChatLoaded { input_buffer, .. } = &mut *state
            {
                if !input_buffer.trim().is_empty() {
                    let msg = input_buffer.trim().to_string();
                    *input_buffer = String::new();
                    drop(state); // Release lock before async operation

                    let msg = msg.into();
                    let _r = sender.broadcast_message(msg).await;
                    match _r {
                        Ok(_r) => {
                            self.clear_input_buffer().await;
                            self.chat_append_msg_history(_r).await;
                            self._notify.notify_waiters();
                        }
                        Err(e) => {
                            tracing::warn!("Error sending message: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    pub async fn handle_key(&self, c: char) {
        let mut state = self.data.lock().await;
        if let ChatPageState::ChatLoaded { input_buffer, .. } = &mut *state {
            input_buffer.push(c);
            self._notify.notify_waiters();
        }
    }

    pub async fn handle_backspace(&self) {
        let mut state = self.data.lock().await;
        if let ChatPageState::ChatLoaded { input_buffer, .. } = &mut *state {
            input_buffer.pop();
            self._notify.notify_waiters();
        }
    }

    async fn chat_set_loading(&self, message: &str) {
        {
            *self.data.lock().await =
                ChatPageState::new_loading(message.to_string());
        }
        self._notify.notify_waiters();
    }
    async fn chat_set_empty(&self, own_identity: NodeIdentity) {
        {
            *self.data.lock().await = ChatPageState::new_chat(own_identity);
        }
        self._notify.notify_waiters();
    }
    async fn chat_set_presence_list(
        &self,
        p2: PresenceList<GlobalChatRoomType>,
    ) {
        {
            self.data.lock().await.chat_set_presence_list(p2);
        }
        self._notify.notify_waiters();
    }
    async fn chat_append_msg_history(
        &self,
        msg: ReceivedMessage<GlobalChatRoomType>,
    ) {
        {
            self.data.lock().await.chat_append_msg_history(msg);
        }
        self._notify.notify_waiters();
    }
    async fn clear_input_buffer(&self) {
        {
            self.data.lock().await.chat_clear_input_buffer();
        }
        self._notify.notify_waiters();
    }

    async fn scroll_messages(&self, delta: i32) {
        let mut state = self.data.lock().await;
        if let ChatPageState::ChatLoaded {
            scroll_position, ..
        } = &mut *state
        {
            *scroll_position =
                (*scroll_position as i32 + delta).max(0) as usize;
            self._notify.notify_waiters();
        }
    }
}

impl ChatPageState {
    fn new_loading(message: String) -> Self {
        Self::ChatLoading { message }
    }
    fn new_chat(own_identity: NodeIdentity) -> Self {
        Self::ChatLoaded {
            own_identity,
            presence: vec![],
            msg_history: vec![],
            input_buffer: String::new(),
            scroll_position: 0,
        }
    }
    fn chat_set_presence_list(&mut self, p2: PresenceList<GlobalChatRoomType>) {
        if let Self::ChatLoaded { presence, .. } = self {
            *presence = p2;
        }
    }
    fn chat_append_msg_history(
        &mut self,
        msg: ReceivedMessage<GlobalChatRoomType>,
    ) {
        if let Self::ChatLoaded {
            msg_history,
            scroll_position,
            ..
        } = self
        {
            msg_history.push(msg);
            *scroll_position = msg_history.len() - 1;
        }
    }
    fn chat_clear_input_buffer(&mut self) {
        if let Self::ChatLoaded { input_buffer, .. } = self {
            *input_buffer = String::new();
        }
    }
}

async fn task_page_driver(page: ChatPage) -> anyhow::Result<()> {
    page.chat_set_loading("Generating identity...").await;
    let id = UserIdentitySecrets::generate();
    page.chat_set_loading("Connecting to server...").await;

    let global_mm = GlobalMatchmaker::new(Arc::new(id)).await?;
    {
        *page.mm.lock().await = Some(global_mm.clone());
    }

    page.chat_set_loading("Connecting to chat...").await;

    let _r = chat_driver(page, global_mm.clone()).await;
    global_mm.shutdown().await?;
    _r
}

async fn chat_driver(
    page: ChatPage,
    global_mm: GlobalMatchmaker,
) -> anyhow::Result<()> {
    let controller = global_mm.global_chat_controller().await.context("F")?;
    let presence = controller.chat_presence();
    let sender = controller.sender();
    sender
        .set_presence(&GlobalChatPresence {
            url: "".to_string(),
            platform: "Terminal UI".to_string(),
            is_server: false,
        })
        .await;
    {
        *page.sender.lock().await = Some(sender);
    }

    page.chat_set_loading("Waiting for chat...").await;
    controller.wait_joined().await?;

    page.chat_set_empty(controller.node_identity()).await;

    let recv = controller.receiver().await;

    loop {
        tokio::select! {
            _ = presence.notified().fuse() => {
                let presence_list = presence.get_presence_list().await;
                page.chat_set_presence_list(presence_list).await;
            }
            msg = recv.next_message().fuse() => {
                let Some(msg) = msg else {
                    anyhow::bail!("Message stream closed");
                };
                page.chat_append_msg_history(msg).await;
            }
        }
    }
}

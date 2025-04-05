mod draw;

use std::sync::Arc;

use crate::terminal_ui::router::{Drawable, DynamicPage, Page, PageFactory};
use anyhow::Context;
use async_trait::async_trait;
use crossterm::event::Event;
use futures::FutureExt;
use n0_future::task::AbortOnDropHandle;
use protocol::{
    chat::{IChatController, IChatReceiver, IChatSender},
    chat_presence::PresenceList,
    global_matchmaker::{
        GlobalChatMessageType, GlobalChatPresence, GlobalMatchmaker,
    },
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

#[derive(Debug, Clone)]
pub struct ChatPage {
    _notify: Arc<Notify>,
    data: Arc<Mutex<ChatPageState>>,
}

#[derive(Debug, Clone)]
pub enum ChatPageState {
    ChatLoaded {
        own_identity: NodeIdentity,
        presence: PresenceList<GlobalChatMessageType>,
        msg_history: Vec<ReceivedMessage<GlobalChatMessageType>>,
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
    async fn shutdown(&self) {}
    async fn handle_event(&self, _event: Event) {}
}

impl Drawable for ChatPageState {
    fn draw(&self, frame: &mut ratatui::Frame) {
        draw::draw_chat(frame, self);
    }
}

impl ChatPage {
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
        p2: PresenceList<GlobalChatMessageType>,
    ) {
        {
            self.data.lock().await.chat_set_presence_list(p2);
        }
        self._notify.notify_waiters();
    }
    async fn chat_append_msg_history(
        &self,
        msg: ReceivedMessage<GlobalChatMessageType>,
    ) {
        {
            self.data.lock().await.chat_append_msg_history(msg);
        }
        self._notify.notify_waiters();
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
        }
    }
    fn chat_set_presence_list(
        &mut self,
        p2: PresenceList<GlobalChatMessageType>,
    ) {
        if let Self::ChatLoaded { presence, .. } = self {
            *presence = p2;
        }
    }
    fn chat_append_msg_history(
        &mut self,
        msg: ReceivedMessage<GlobalChatMessageType>,
    ) {
        if let Self::ChatLoaded { msg_history, .. } = self {
            msg_history.push(msg);
        }
    }
}

async fn task_page_driver(page: ChatPage) -> anyhow::Result<()> {
    page.chat_set_loading("Generating identity...").await;
    let id = UserIdentitySecrets::generate();
    page.chat_set_loading("Connecting to server...").await;

    let global_mm = GlobalMatchmaker::new(Arc::new(id)).await?;

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
    let sender = controller.sender();

    sender
        .set_presence(&GlobalChatPresence {
            url: "".to_string(),
            platform: "Terminal UI".to_string(),
        })
        .await;

    page.chat_set_loading("Waiting for chat...").await;
    controller.wait_joined().await?;

    page.chat_set_empty(controller.node_identity()).await;

    let presence = controller.chat_presence();
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

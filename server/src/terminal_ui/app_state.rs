use std::sync::Arc;

use protocol::user_identity::NodeIdentity;
use protocol::ReceivedMessage;
use protocol::{
    chat_presence::PresenceList, global_matchmaker::GlobalChatMessageType,
};
use tokio::sync::{futures::Notified, Mutex, Notify};
use tracing::warn;

#[derive(Clone, Debug)]
pub struct LoadingWindowData {
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct ChatWindowData {
    pub own_identity: NodeIdentity,
    pub presence: PresenceList<GlobalChatMessageType>,
    pub msg_history: Vec<ReceivedMessage<GlobalChatMessageType>>,
}

#[derive(Clone, Debug)]
pub enum WindowData {
    Loading(LoadingWindowData),
    Chat(ChatWindowData),
}

#[derive(Clone, Debug)]
pub struct AppState {
    window_data: Arc<Mutex<WindowData>>,
    window_notify: Arc<Notify>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            window_data: Arc::new(Mutex::new(WindowData::Loading(
                LoadingWindowData {
                    message: "Loading...".to_string(),
                },
            ))),
            window_notify: Arc::new(Notify::new()),
        }
    }
    pub async fn set_state(&self, state: WindowData) {
        let mut window_data = self.window_data.lock().await;
        *window_data = state;
        self.window_notify.notify_waiters();
    }
    pub async fn set_presence_list(
        &self,
        presence_list: PresenceList<GlobalChatMessageType>,
    ) {
        let mut window_data = self.window_data.lock().await;
        if let WindowData::Chat(chat_data) = &mut *window_data {
            chat_data.presence = presence_list;
        } else {
            warn!("AppState::set_presence_list: WindowData is not Chat");
        }
        self.window_notify.notify_waiters();
    }
    pub async fn append_msg_history(
        &self,
        msg: ReceivedMessage<GlobalChatMessageType>,
    ) {
        let mut window_data = self.window_data.lock().await;
        if let WindowData::Chat(chat_data) = &mut *window_data {
            chat_data.msg_history.push(msg);
        } else {
            warn!("AppState::append_msg_history: WindowData is not Chat");
        }
        self.window_notify.notify_waiters();
    }
    pub async fn set_loading_message(&self, message: &str) {
        self.set_state(WindowData::Loading(LoadingWindowData {
            message: message.to_string(),
        }))
        .await;
    }
    pub async fn get_state(&self) -> WindowData {
        let window_data = self.window_data.lock().await;
        window_data.clone()
    }
    pub fn notified(&self) -> Notified {
        self.window_notify.notified()
    }
    pub async fn should_exit(&self) -> bool {
        false
    }
}

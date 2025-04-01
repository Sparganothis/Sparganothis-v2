use std::sync::Arc;

use crate::{
    _const::CONNECT_TIMEOUT, signed_message::AcceptableType,
    sleep::SleepManager,
};
use iroh::{
    endpoint::Connection, protocol::ProtocolHandler, Endpoint, PublicKey,
};
use iroh_gossip::proto::TopicId;
use n0_future::task::spawn;
use n0_future::task::AbortOnDropHandle;
use tokio::sync::Mutex;
use tracing::info;
use tracing::warn;

pub const CHAT_DIRECT_MESSAGE_ALPN: &[u8] = b"/chat-direct-message/0";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatDirectMessage(pub TopicId, pub Arc<Vec<u8>>);

#[derive(Debug, Clone)]
pub struct DirectMessageProtocol<T> {
    received_message_broadcaster: async_broadcast::Sender<(PublicKey, T)>,
    sleep_manager: SleepManager,
    _endpoint: Endpoint,
    _task: Arc<Mutex<Option<AbortOnDropHandle<anyhow::Result<()>>>>>,
    sender_tx: tokio::sync::mpsc::Sender<(PublicKey, T)>,
}

impl<T: AcceptableType> DirectMessageProtocol<T> {
    pub async fn shutdown(&self) {
        let mut task = self._task.lock().await;
        if let Some(_task) = task.take() {
            info!("shutting down direct message sender");
            drop(_task);
            self.received_message_broadcaster.close();
        }
    }
    pub fn new(
        received_message_broadcaster: async_broadcast::Sender<(PublicKey, T)>,
        sleep_manager: SleepManager,
        endpoint: Endpoint,
    ) -> Self {
        let (sender_tx, mut sender_rx) = tokio::sync::mpsc::channel(16);
        let _endpoint = endpoint.clone();
        let task = async move {
            while let Some((iroh_target, payload)) = sender_rx.recv().await {
                if let Err(e) = Self::do_send_direct_message(
                    &_endpoint,
                    iroh_target,
                    payload,
                )
                .await
                {
                    warn!("failed to send direct message: {:?}", e);
                }
            }
            warn!("direct message sender task closed");
            Ok(())
        };
        let task = AbortOnDropHandle::new(spawn(task));
        let task = Arc::new(Mutex::new(Some(task)));
        Self {
            received_message_broadcaster,
            sleep_manager,
            _endpoint: endpoint,
            _task: task,
            sender_tx,
        }
    }

    async fn handle_connection(
        self,
        connection: Connection,
    ) -> anyhow::Result<()> {
        let _remote_node_id = connection.remote_node_id()?;
        let mut recv = connection.accept_uni().await?;
        let data = recv.read_to_end(1024 * 63).await?;
        connection.close(0u8.into(), b"done");
        let data = postcard::from_bytes(&data)?;
        self.received_message_broadcaster
            .broadcast((_remote_node_id, data))
            .await?;
        self.sleep_manager.wake_up();
        Ok(())
    }

    pub async fn send_direct_message(
        &self,
        iroh_target: PublicKey,
        payload: T,
    ) -> anyhow::Result<()> {
        self.sender_tx.send((iroh_target, payload)).await?;
        Ok(())
    }

    async fn do_send_direct_message(
        endpoint: &Endpoint,
        iroh_target: PublicKey,
        payload: T,
    ) -> anyhow::Result<()> {
        let connection = n0_future::time::timeout(
            CONNECT_TIMEOUT,
            endpoint.connect(iroh_target, CHAT_DIRECT_MESSAGE_ALPN),
        )
        .await??;
        let payload = postcard::to_stdvec(&payload)?;
        let mut send_stream = connection.open_uni().await?;
        send_stream.write_all(&payload).await?;
        send_stream.finish()?;
        connection.closed().await;
        // connection.close(0u8.into(), b"done");
        Ok(())
    }
}

impl<T: AcceptableType> ProtocolHandler for DirectMessageProtocol<T> {
    fn accept(
        &self,
        connection: Connection,
    ) -> n0_future::boxed::BoxFuture<anyhow::Result<()>> {
        Box::pin(self.clone().handle_connection(connection))
    }
}

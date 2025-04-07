use std::collections::HashMap;
use std::sync::Arc;

use crate::_const::PRESENCE_EXPIRATION;
use crate::{
    _const::CONNECT_TIMEOUT, signed_message::AcceptableType,
    sleep::SleepManager,
};
use anyhow::Context;
use iroh::{
    endpoint::Connection, protocol::ProtocolHandler, Endpoint, PublicKey,
};
use iroh_gossip::proto::TopicId;
use n0_future::task::spawn;
use n0_future::task::AbortOnDropHandle;
use tokio::sync::RwLock;
use tracing::info;
use tracing::warn;

pub const CHAT_DIRECT_MESSAGE_ALPN: &[u8] = b"/chat-direct-message/0";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatDirectMessage(pub TopicId, pub Arc<Vec<u8>>);

#[derive(Debug, Clone)]
pub struct DirectMessageProtocol<T> {
    received_message_broadcaster: async_broadcast::Sender<(PublicKey, T)>,
    sleep_manager: SleepManager,
    _task: Arc<RwLock<Option<AbortOnDropHandle<anyhow::Result<()>>>>>,
    sender_tx: tokio::sync::mpsc::Sender<(PublicKey, T)>,
    message_dispatchers: MessageDispatchers<T>,
}

impl<T: AcceptableType> DirectMessageProtocol<T> {
    pub async fn shutdown(&self) {
        self.message_dispatchers.shutdown().await;
        let mut task = self._task.write().await;
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
        let msg_d = MessageDispatchers::new(endpoint);
        let _msg_d2 = msg_d.clone();
        let task = async move {
            while let Some((iroh_target, payload)) = sender_rx.recv().await {
                if let Err(_e) =
                    _msg_d2.send_message(iroh_target, payload).await
                {
                    // warn!("failed to send direct message: {:?}", _e);
                    // warn!("dropping dispatcher for {}", iroh_target);
                    _msg_d2.drop_dispatcher(iroh_target).await;
                }
            }
            warn!("direct message sender task closed");
            Ok(())
        };
        let task = AbortOnDropHandle::new(spawn(task));
        let task = Arc::new(RwLock::new(Some(task)));
        Self {
            received_message_broadcaster,
            sleep_manager,
            _task: task,
            sender_tx,
            message_dispatchers: msg_d,
        }
    }

    async fn handle_connection(
        self,
        connection: Connection,
    ) -> anyhow::Result<()> {
        let _remote_node_id = connection.remote_node_id()?;
        let mut recv = connection.accept_uni().await?;
        loop {
            let mut data_len = [0; 4];
            recv.read_exact(&mut data_len).await?;
            let data_len = u32::from_le_bytes(data_len);
            let mut data = vec![0; data_len as usize];
            recv.read_exact(&mut data).await?;
            let data = postcard::from_bytes(&data)?;
            self.received_message_broadcaster
                .broadcast((_remote_node_id, data))
                .await?;
            self.sleep_manager.wake_up();
        }
    }

    pub async fn send_direct_message(
        &self,
        iroh_target: PublicKey,
        payload: T,
    ) -> anyhow::Result<()> {
        self.sender_tx.send((iroh_target, payload)).await?;
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

#[derive(Debug, Clone)]
struct MessageDispatchers<T> {
    endpoint: Endpoint,
    dispatchers: Arc<RwLock<HashMap<PublicKey, Arc<MessageDispatcher<T>>>>>,
}

impl<T: AcceptableType> MessageDispatchers<T> {
    pub fn new(endpoint: Endpoint) -> Self {
        info!("creating message dispatchers dict");
        Self {
            endpoint,
            dispatchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub async fn shutdown(&self) {
        info!("shutting down message dispatchers dict");
        let mut dispatchers = self.dispatchers.write().await;
        dispatchers.clear();
    }
    async fn ensure_dispatcher(
        &self,
        target: PublicKey,
    ) -> Arc<MessageDispatcher<T>> {
        let mut dispatchers = self.dispatchers.write().await;
        if let Some(dispatcher) = dispatchers.get_mut(&target) {
            return dispatcher.clone();
        }

        let dispatcher =
            Arc::new(MessageDispatcher::new(target, self.endpoint.clone()));
        dispatchers.insert(target, dispatcher.clone());
        dispatcher
    }
    pub async fn drop_dispatcher(&self, target: PublicKey) {
        let mut dispatchers = self.dispatchers.write().await;
        dispatchers.remove(&target);
    }
    pub async fn send_message(
        &self,
        target: PublicKey,
        payload: T,
    ) -> anyhow::Result<()> {
        let dispatcher = self.ensure_dispatcher(target).await;
        dispatcher.send_message(payload).await
    }
}

#[derive(Debug)]
struct MessageDispatcher<T> {
    sender: tokio::sync::mpsc::Sender<T>,
    _task: AbortOnDropHandle<anyhow::Result<()>>,
}

impl<T: AcceptableType> MessageDispatcher<T> {
    pub fn new(target: PublicKey, endpoint: Endpoint) -> Self {
        // info!("creating message dispatcher for {}", target);
        let (sender, mut receiver) = tokio::sync::mpsc::channel(16);
        let _task = async move {
            let connection = n0_future::time::timeout(
                CONNECT_TIMEOUT,
                endpoint.connect(target, CHAT_DIRECT_MESSAGE_ALPN),
            )
            .await??;
            let mut send_stream = n0_future::time::timeout(
                CONNECT_TIMEOUT,
                connection.open_uni(),
            )
            .await??;

            while let Some(payload) =
                n0_future::time::timeout(PRESENCE_EXPIRATION, receiver.recv())
                    .await
                    .context("no direct message requested, exiting")?
            {
                let payload = postcard::to_stdvec(&payload)?;
                let len = (payload.len() as u32).to_le_bytes();
                n0_future::time::timeout(
                    CONNECT_TIMEOUT,
                    send_stream.write_all(&len),
                )
                .await??;
                n0_future::time::timeout(
                    CONNECT_TIMEOUT,
                    send_stream.write_all(&payload),
                )
                .await??;
            }

            n0_future::time::timeout(CONNECT_TIMEOUT, async move {
                send_stream.finish()?;
                connection.closed().await;
                anyhow::Ok(())
            })
            .await??;

            anyhow::Ok(())
        };
        let _task = async move {
            let _r = _task.await;
            // info!("direct message dispatcher for {} closed!!", target);
            // message_dispatchers.drop_dispatcher(target).await;
            _r
        };
        let _task = AbortOnDropHandle::new(spawn(_task));
        Self { sender, _task }
    }
    pub async fn send_message(&self, payload: T) -> anyhow::Result<()> {
        self.sender.send(payload).await?;
        Ok(())
    }
}

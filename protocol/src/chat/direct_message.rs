use std::sync::Arc;

use iroh::{
    endpoint::Connection, protocol::ProtocolHandler, Endpoint, PublicKey,
};
use iroh_gossip::proto::TopicId;

use crate::{
    _const::CONNECT_TIMEOUT,
    signed_message::{
        AcceptableType, MessageSigner, SignedMessage, WireMessage,
    },
    sleep::SleepManager,
};

pub const CHAT_DIRECT_MESSAGE_ALPN: &[u8] = b"/chat-direct-message/0";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ChatDirectMessage (pub TopicId, pub Arc<Vec<u8>>);

#[derive(Debug, Clone)]
pub struct DirectMessageProtocol<T> {
    pub received_message_broadcaster: async_broadcast::Sender<(PublicKey, WireMessage<T>)>,
    pub sleep_manager: SleepManager,
    pub endpoint: Endpoint,
}

impl<T: AcceptableType> DirectMessageProtocol<T> {
    async fn handle_connection(
        self,
        connection: Connection,
    ) -> anyhow::Result<()> {
        let _remote_node_id = connection.remote_node_id()?;
        let mut recv = connection.accept_uni().await?;
        let data = recv.read_to_end(1024 * 63).await?;
        connection.close(0u8.into(), b"done");
        let data = SignedMessage::verify_and_decode(&data)?;
        if data.from.node_id() != &_remote_node_id {
            return Err(anyhow::anyhow!("node id mismatch"));
        }
        self.received_message_broadcaster.broadcast((_remote_node_id, data)).await?;
        self.sleep_manager.wake_up();
        Ok(())
    }

    pub async fn send_direct_message(
        &self,
        iroh_target: PublicKey,
        payload: T,
        message_signer: &MessageSigner,
    ) -> anyhow::Result<()> {
        let connection = n0_future::time::timeout(
            CONNECT_TIMEOUT,
            self.endpoint.connect(iroh_target, CHAT_DIRECT_MESSAGE_ALPN),
        )
        .await??;
        let payload = message_signer.sign_and_encode(payload)?;
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


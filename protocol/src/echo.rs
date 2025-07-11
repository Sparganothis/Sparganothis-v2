use anyhow::{Context, Result};
use iroh::{endpoint::Connection, protocol::ProtocolHandler, NodeId};
use n0_future::boxed::BoxFuture;
use tracing::warn;

use crate::{_const::CONNECT_TIMEOUT, sleep::SleepManager};

#[derive(Debug, Clone)]
pub struct Echo {
    own_endpoint_node_id: NodeId,
    sleep_manager: SleepManager,
}

impl Echo {
    pub const ALPN: &[u8] = b"sparganothis/global-matchmaker-echo/0";
    pub fn new(
        own_endpoint_node_id: NodeId,
        sleep_manager: SleepManager,
    ) -> Self {
        Self {
            own_endpoint_node_id,
            sleep_manager,
        }
    }
}

impl ProtocolHandler for Echo {
    /// The `accept` method is called for each incoming connection for our ALPN.
    ///
    /// The returned future runs on a newly spawned tokio task, so it can run as long as
    /// the connection lasts.
    fn accept(&self, connection: Connection) -> BoxFuture<Result<()>> {
        Box::pin(self.clone().handle_connection(connection))
    }
}

impl Echo {
    async fn handle_connection(self, connection: Connection) -> Result<()> {
        // We can get the remote's node id from the connection.
        self.sleep_manager.wake_up();
        let res = self.handle_connection2(&connection).await;
        if let Err(e) = res.as_ref() {
            warn!("Failed to handle connection: {e}");
        }

        res
    }
    async fn handle_connection2(&self, connection: &Connection) -> Result<()> {
        // We can get the remote's node id from the connection.
        let response_own_node_id = self.own_endpoint_node_id.as_bytes().clone();

        // Our protocol is a simple request-response protocol, so we expect the
        // connecting peer to open a single bi-directional stream.
        let (mut send, mut recv) = connection.accept_bi().await?;

        let mut recv_buf = vec![0; 32];
        n0_future::time::timeout(
            CONNECT_TIMEOUT,
            recv.read_exact(&mut recv_buf),
        )
        .await
        .context("echo")?
        .context("echo")?;
        if recv_buf != connection.remote_node_id()?.as_bytes().to_vec() {
            return Err(anyhow::anyhow!("Invalid node id"));
        }

        n0_future::time::timeout(
            CONNECT_TIMEOUT,
            send.write_all(&response_own_node_id),
        )
        .await
        .context("echo")?
        .context("echo")?;

        // By calling `finish` on the send stream we signal that we will not send anything
        // further, which makes the receive stream on the other end terminate.
        send.finish()?;

        // Wait until the remote closes the connection, which it does once it
        // received the response.
        connection.closed().await;
        Ok(())
    }
}

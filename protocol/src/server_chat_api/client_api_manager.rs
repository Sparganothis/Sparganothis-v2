use std::sync::Arc;

use n0_future::task::AbortOnDropHandle;
use rand::Rng;
use tokio::sync::RwLock;

use crate::{
    chat::{ChatController, IChatController, IChatReceiver, IChatSender},
    global_matchmaker::GlobalMatchmaker,
    server_chat_api::{
        api_method_macros::ApiMethod,
        join_chat::{
            client_join_server_chat, fetch_server_ids,
            ServerChatMessageContent, ServerChatRoomType,
        },
    },
    user_identity::NodeIdentity,
};

#[derive(Debug, Clone)]
pub struct ClientApiManager {
    chat_controller: ChatController<ServerChatRoomType>,
    server_identity: Arc<RwLock<NodeIdentity>>,
    _main_loop: Arc<AbortOnDropHandle<anyhow::Result<()>>>,
}

impl PartialEq for ClientApiManager {
    fn eq(&self, other: &Self) -> bool {
        self.chat_controller == other.chat_controller
    }
}

pub async fn connect_api_manager(
    mm: GlobalMatchmaker,
) -> anyhow::Result<ClientApiManager> {
    let (nodes, chat_controller) = client_join_server_chat(mm.clone()).await?;
    let node_idx = (&mut rand::thread_rng()).gen_range(0..nodes.len());
    let mut node = nodes[node_idx];
    let server_identity = Arc::new(RwLock::new(node));

    let mm2 = mm.clone();
    let si2 = server_identity.clone();
    let cc2 = chat_controller.clone();
    let main_loop =
        Arc::new(AbortOnDropHandle::new(n0_future::task::spawn(async move {
            loop {
                let Some(gcc) = mm.global_chat_controller().await else {
                    mm2.sleep(std::time::Duration::from_secs(3)).await;
                    tracing::warn!("no gloabl chat controller!");
                    continue;
                };
                let presence = gcc.chat_presence();
                let _n = presence.notified().await;
                mm2.sleep(std::time::Duration::from_secs(2)).await;

                let server_presence =
                    fetch_server_ids(mm2.clone()).await.unwrap_or(vec![]);
                if server_presence.is_empty() {
                    tracing::warn!("CLIENT FOUND NO SERVERS TO TALK WITH!");
                    continue;
                }
                let old_node = node;
                if server_presence.contains(&old_node) {
                    continue;
                }

                let node_idx =
                    (&mut rand::thread_rng()).gen_range(0..nodes.len());
                node = server_presence[node_idx];
                {
                    if let Err(e) =
                        cc2.sender().join_peers(vec![*node.node_id()]).await
                    {
                        tracing::error!(
                            "error joining newly selected peer: {e}"
                        );
                    };
                    let mut w = si2.write().await;
                    *w = node;
                }
                tracing::warn!("switched server from {old_node:?} to {node:?}");
            }
        })));

    Ok(ClientApiManager {
        chat_controller,
        server_identity,
        _main_loop: main_loop,
    })
}

pub const API_METHOD_TIMEOUT_SECONDS: f32 = 5.0;

impl ClientApiManager {
    pub async fn call_method<M: ApiMethod>(
        &self,
        arg: M::Arg,
    ) -> anyhow::Result<M::Ret> {
                tracing::info!("vvv\ncall start method={} \n arg: {:#?} \n^^^/n", M::NAME, &arg);

        let ret = n0_future::time::timeout(
            std::time::Duration::from_secs_f32(API_METHOD_TIMEOUT_SECONDS),
            self._do_call_method::<M>(arg.clone()),
        )
        .await;
        tracing::info!("vvv\ncall result method={} \n arg: {:#?} \nret: {:#?} \n^^^\n", M::NAME, &arg, &ret);
        ret?
    }

    async fn _do_call_method<M: ApiMethod>(
        &self,
        arg: M::Arg,
    ) -> anyhow::Result<M::Ret> {
        let arg_v = postcard::to_stdvec(&arg)?;

        let cc = self.chat_controller.clone();
        let sender = cc.sender();

        let nonce = (&mut rand::thread_rng()).gen::<i64>();
        let method_name = M::NAME.to_string();

        let request_message = ServerChatMessageContent::Request {
            method_name: method_name.clone(),
            nonce: nonce,
            req: arg_v,
        };
        let receiver = cc.receiver().await;
        let server_identity = { self.server_identity.read().await.clone() };
        tracing::info!(
            "Sending direct message for method {method_name} nonce={nonce}"
        );
        sender
            .direct_message(server_identity, request_message)
            .await?;

        while let Some(reply_message) = receiver.next_message().await {
            let reply = reply_message.message;

            let ServerChatMessageContent::Reply {
                method_name: r_method_name,
                nonce: r_nonce,
                ret: ret_bytes,
            } = reply
            else {
                tracing::warn!("reply is not reply!");
                continue;
            };
            if r_method_name != method_name || r_nonce != nonce {
                tracing::warn!("Received unrelated message for this call: {r_method_name} {r_nonce}");
                continue;
            }
            let Ok(ret_bytes) = ret_bytes else {
                let err = ret_bytes.unwrap_err();
                tracing::warn!("error: {:?}", err);
                anyhow::bail!("got error: {err}");
            };
            tracing::info!("\n Got back message with reply for method: \n {method_name} {nonce} : length = {}", ret_bytes.len());
            let ret = postcard::from_bytes(&ret_bytes)?;
            return Ok(ret);
        }
        tracing::warn!("No more messages in chat!");
        anyhow::bail!("no more messages in chat!");
    }
}

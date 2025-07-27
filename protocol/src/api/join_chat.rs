use std::{
    collections::{BTreeSet, HashSet},
    time::Duration,
};

use anyhow::Context;
use n0_future::task::AbortOnDropHandle;
use serde::{Deserialize, Serialize};

use crate::{
    api::api_const::API_SERVER_VERSION,
    chat::{
        chat_controller::{
            ChatController, IChatController, IChatReceiver, IChatSender,
        },
        chat_ticket::ChatTicket,
        global_chat::GlobalChatMessageContent,
    },
    global_matchmaker::GlobalMatchmaker,
    user_identity::NodeIdentity,
    IChatRoomType,
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ServerChatRoomType;

impl IChatRoomType for ServerChatRoomType {
    type M = ServerChatMessageContent;
    type P = ServerChatPresence;
    fn default_presence() -> Self::P {
        ServerChatPresence::default()
    }
}
#[derive(
    Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize, Default,
)]
pub struct ServerChatPresence {
    pub is_server: bool,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ServerChatMessageContent {
    Request {
        method_name: String,
        nonce: i64,
        req: Vec<u8>,
    },
    Reply {
        method_name: String,
        nonce: i64,
        ret: Result<Vec<u8>, String>,
    },
}

pub async fn server_join_server_chat(
    mm: GlobalMatchmaker,
) -> anyhow::Result<ChatController<ServerChatRoomType>> {
    let Some(nn) = mm.own_node().await else {
        anyhow::bail!("server_join_server_chat: no node!");
    };
    let chat_ticket = ChatTicket::new_str_bs("server", BTreeSet::from([]));
    let Ok(chat) = nn.join_chat::<ServerChatRoomType>(&chat_ticket).await
    else {
        anyhow::bail!("server_join_server_chat: Failed to join server chat");
    };
    Ok(chat)
}

async fn client_join_server_chat_with_server_ids(
    mm: GlobalMatchmaker,
    server_nodes: Vec<NodeIdentity>,
) -> anyhow::Result<ChatController<ServerChatRoomType>> {
    if server_nodes.is_empty() {
        anyhow::bail!("client_join_server_chat: no server nodes!!");
    }
    let Some(nn) = mm.own_node().await else {
        anyhow::bail!("client_join_server_chat: no node!");
    };
    let chat_ticket = ChatTicket::new_str_bs(
        "server",
        BTreeSet::from_iter(server_nodes.iter().map(|x| *x.node_id())),
    );
    let Ok(chat) = nn.join_chat::<ServerChatRoomType>(&chat_ticket).await
    else {
        anyhow::bail!("client_join_server_chat: Failed to join server chat");
    };
    Ok(chat)
}

pub(crate) async fn fetch_server_ids(
    mm: GlobalMatchmaker,
) -> anyhow::Result<Vec<NodeIdentity>> {
    tracing::info!("fetch_server_ids()");
    let global = mm
        .global_chat_controller()
        .await
        .context("no global chat?")?;
    let presence = global.chat_presence();
    let mut bs_sent_to = HashSet::new();
    let req = GlobalChatMessageContent::BootstrapQuery(
        crate::chat::global_chat::GlobalChatBootstrapQuery::PlzSendServerList,
    );

    let rr = global.receiver().await;
    let pp = global.chat_presence();
    let fetch_task = AbortOnDropHandle::new(n0_future::task::spawn(
        async move {
            while let Some(msg1) = rr.next_message().await {
                let msg = msg1.message;
                let GlobalChatMessageContent::BootstrapQuery(crate::chat::global_chat::GlobalChatBootstrapQuery::ServerList { v }) = msg else {
                continue;
            };
                for x in v.0 {
                    pp.add_presence(&x.identity, &x.payload).await;
                }
            }
        },
    ));

    let mut server_nodes: Vec<_> = vec![];
    for _retry in 0..10 {
        mm.sleep(Duration::from_millis(16 + _retry)).await;
        if _retry == 2 {
            tracing::info!("Broadcasting request for server list!");
            let _ = global.sender().broadcast_message(req.clone()).await;
        }

        let presence_list = presence.get_presence_list().await;
        for p in presence_list.0 {
            let Some(payload) = &p.payload else {
                continue;
            };
            let node_id = p.identity;
            if let Some(_idx) = node_id.bootstrap_idx() {
                if !bs_sent_to.contains(&node_id) {
                    bs_sent_to.insert(node_id);
                    tracing::info!(
                        "Sending individual message for server list!"
                    );
                    let _ = global
                        .sender()
                        .direct_message(node_id, req.clone())
                        .await;
                    continue;
                }
            }

            let Some(server_info) = payload.is_server.clone() else {
                continue;
            };
            if server_info.server_version != API_SERVER_VERSION {
                continue;
            }
            server_nodes.push(node_id);
        }
        if server_nodes.len() >= 1 {
            break;
        }

        mm.sleep(Duration::from_millis(60 + _retry)).await;
    }
    // fetch_task.abort();
    drop(fetch_task);

    Ok(server_nodes)
}

pub(crate) async fn client_join_server_chat(
    mm: GlobalMatchmaker,
) -> anyhow::Result<(Vec<NodeIdentity>, ChatController<ServerChatRoomType>)> {
    const RETRY_COUNT: i32 = 8;
    const RETRY_SLEEP_SECONDS: i32 = 1;

    for i in 0..=RETRY_COUNT {
        tracing::info!("connecting to server chat {i}/{RETRY_COUNT} ... ");
        let server_nodes = fetch_server_ids(mm.clone()).await.unwrap_or(vec![]);

        if server_nodes.is_empty() {
            tracing::warn!("FOUND NO SERVER NODES!");
            let sleep = i + RETRY_SLEEP_SECONDS;
            n0_future::time::sleep(Duration::from_secs(sleep as u64)).await;
            continue;
        }

        let chat = client_join_server_chat_with_server_ids(
            mm.clone(),
            server_nodes.clone(),
        )
        .await;
        if let Ok(chat) = chat {
            if let Err(e) = chat.wait_joined().await {
                tracing::warn!(
                    "retry error {i}/{RETRY_COUNT}: on wait_joined: {e}"
                );
            }

            tracing::info!("server chat OK.");

            return Ok((server_nodes, chat));
        } else {
            if i == RETRY_COUNT {
                tracing::error!("final error: {:#?}", chat);
                anyhow::bail!("{chat:#?}")
            } else {
                tracing::warn!("retry error {i}/{RETRY_COUNT}: {:?}", chat);

                let sleep = i + RETRY_SLEEP_SECONDS;
                n0_future::time::sleep(Duration::from_secs(sleep as u64)).await;
            }
        }
    }
    anyhow::bail!("failed to join server chat with existing server.")
}

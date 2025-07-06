use std::{collections::BTreeSet, time::Duration};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{
    chat::{ChatController, IChatController},
    chat_ticket::ChatTicket,
    global_matchmaker::GlobalMatchmaker,
    server_chat_api::api_methods::SERVER_VERSION,
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
    let global = mm
        .global_chat_controller()
        .await
        .context("no global chat?")?;
    let presence = global.chat_presence();

    let presence_list = presence.get_presence_list().await;
    let mut server_nodes: Vec<_> = vec![];
    for p in presence_list {
        let Some(payload) = &p.payload else {
            continue;
        };
        let node_id = p.identity;
        let Some(server_info) = payload.is_server.clone() else {
            continue;
        };
        if server_info.server_version != SERVER_VERSION {
            continue;
        }
        server_nodes.push(node_id);
    }

    Ok(server_nodes)
}

pub(crate) async fn client_join_server_chat(
    mm: GlobalMatchmaker,
) -> anyhow::Result<(Vec<NodeIdentity>, ChatController<ServerChatRoomType>)> {
    const RETRY_COUNT: i32 = 8;
    const RETRY_SLEEP_SECONDS: i32 = 2;

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

use protocol::{
    chat::{IChatController, IChatReceiver, IChatSender},
    global_chat::{GlobalChatMessageContent, GlobalChatPresence},
    global_matchmaker::GlobalMatchmaker,
    server_chat_api::{api_methods::{inventory_get_implementation_by_name, ServerInfo, SERVER_VERSION}, join_chat::{
        server_join_server_chat, ServerChatMessageContent, ServerChatPresence,
        ServerChatRoomType
    }},
    user_identity::NodeIdentity,
    ReceivedMessage,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::info;

pub async fn server_main_loop(global_mm: GlobalMatchmaker, server_name: String) -> anyhow::Result<()> {
    // let mut our_ticket = ticket.clone();
    // our_ticket.bootstrap = [node.node_id()].into_iter().collect();
    //  tracing::info!("* ticket to join this chat:");
    //  tracing::info!("{}", our_ticket.serialize());

     tracing::info!("* waiting for peers ...");
    let global_controller = global_mm.global_chat_controller().await.unwrap();
    let global_sender = global_controller.sender();
    let global_receiver = global_controller.receiver().await;

    global_sender
        .set_presence(&GlobalChatPresence {
            url: "".to_string(),
            platform: format!("server v{SERVER_VERSION}"),
            is_server: Some(ServerInfo {
                server_version: SERVER_VERSION,
                server_name
            }),
        })
        .await;

    // controller.wait_joined().await?;

     tracing::info!("***********************************************************");
     tracing::info!("* join OK");
     tracing::info!("***********************************************************");
     tracing::info!("> ");

    let global_receive = tokio::task::spawn(async move {
        while let Some(message) = global_receiver.next_message().await {
            let nickname = message.from.nickname();
            let node_id = message.from.node_id().fmt_short();
            let user_id = message.from.user_id().fmt_short();
            match message.message {
                GlobalChatMessageContent::TextMessage { text } => {
                     tracing::info!("<{user_id}@{node_id}> {nickname}: {text}");
                }
                GlobalChatMessageContent::MatchmakingMessage { .. } => {
                     tracing::info!("{nickname} matchmaking: 1v1 message!");
                }
                _ => {
                     tracing::info!("<{user_id}@{node_id}> {nickname}: other message: {:#?}", message.message)
                }
            }
        }
         tracing::info!("* recv closed");
        anyhow::Ok(())
    });

    let global_send = tokio::task::spawn(async move {
        let mut input = BufReader::new(tokio::io::stdin()).lines();
        while let Some(line) = input.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
             tracing::info!("* sending message: {line}");
            global_sender
                .broadcast_message(line.to_string().into())
                .await?;
        }
         tracing::info!("* sender closed.");
        anyhow::Ok(())
    });

    // /////////////////////////
    //
    let server_chat = server_join_server_chat(global_mm.clone()).await?;
    let server_sender = server_chat.sender();
    server_sender
        .set_presence(&ServerChatPresence { is_server: true })
        .await;
    let server_receiver = server_chat.receiver().await;

    let server_recv_thread = n0_future::task::spawn(async move {
        while let Some(message) = server_receiver.next_message().await {
            let Some((to, message)) = server_process_message(message).await else {
                continue;
            };
            let _r = server_sender.direct_message(to, message).await;
            if _r.is_err() {
                tracing::error!("erroR: {_r:#?}");
            }
        }
        Ok(())
    });

    // /////////////////////////
    //    INIT

    let _r1 = n0_future::future::race(
        n0_future::future::race(global_receive, global_send),
        server_recv_thread,
    )
    .await?;

    info!("* SERVER server_main_loop closed.");

    Ok(())
}

async fn server_process_message(
    message: ReceivedMessage<ServerChatRoomType>,
) -> Option<(NodeIdentity, ServerChatMessageContent)> {
    let from = message.from;
    let message = message.message;
    let ServerChatMessageContent::Request{method_name, nonce, req} = message else {
        return None;
    };

    let reply = server_compute_reply(from, method_name, nonce, req).await;

    Some((from, reply))
}

async fn server_compute_reply(
    from: NodeIdentity,
    method_name: String,
    nonce: i64,
    req: Vec<u8>,
) -> ServerChatMessageContent {
    // Ok(match request {
    //     ServerMessageRequest::GuestLoginMessage {} => {
    //         // db_add_guest_login(from).await?;
    //         ServerMessageReply::GuestLoginMessage {}
    //     }
    // })
    let Ok(function) = inventory_get_implementation_by_name(&method_name) else {
        return ServerChatMessageContent::Reply { method_name, nonce, ret: Err("method not found".to_string()) };
    };
    let future = tokio::task::spawn((function.func)(from, req));
    let ret = future.await.map_err(|e| format!("{e:#?}"));
    let ret = match ret {
        Ok(ret) => ret,
        Err(e) => Err(e),
    };
    ServerChatMessageContent::Reply {
        method_name,
        nonce,
        ret
    }
}

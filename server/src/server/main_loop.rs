use std::{collections::BTreeSet, sync::Arc};

use anyhow::{Context, Result};
use protocol::{
    chat::{ChatController, IChatController, IChatReceiver, IChatSender}, chat_ticket::ChatTicket, global_chat::{GlobalChatMessageContent, GlobalChatPresence}, global_matchmaker::GlobalMatchmaker, server_chat::{ServerChatMessageContent, ServerMessageReply, ServerMessageRequest}, user_identity::{NodeIdentity, UserIdentitySecrets}, IChatRoomType, ReceivedMessage
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::info;
use protocol::server_chat::{ServerChatPresence, ServerChatRoomType};

use crate::server::db::guest_login::db_add_guest_login;


pub async fn server_main_loop(global_mm: GlobalMatchmaker) -> Result<()> {
    // let mut our_ticket = ticket.clone();
    // our_ticket.bootstrap = [node.node_id()].into_iter().collect();
    // println!("* ticket to join this chat:");
    // println!("{}", our_ticket.serialize());

    println!("* waiting for peers ...");
    let global_controller = global_mm.global_chat_controller().await.unwrap();
    let global_sender = global_controller.sender();
    let global_receiver = global_controller.receiver().await;

    global_sender
        .set_presence(&GlobalChatPresence {
            url: "".to_string(),
            platform: "server".to_string(),
            is_server: true,
        })
        .await;

    // controller.wait_joined().await?;

    println!("***********************************************************");
    println!("* join OK");
    println!("***********************************************************");
    println!("> ");

    let global_receive = tokio::task::spawn(async move {
        while let Some(message) = global_receiver.next_message().await {
            let nickname = message.from.nickname();
            let node_id = message.from.node_id().fmt_short();
            let user_id = message.from.user_id().fmt_short();
            match message.message {
                GlobalChatMessageContent::TextMessage { text } => {
                    println!("<{user_id}@{node_id}> {nickname}: {text}");
                }
                GlobalChatMessageContent::MatchmakingMessage { .. } => {
                    println!("{nickname} matchmaking: 1v1 message!");
                }
                _ => {
                    println!("<{user_id}@{node_id}> {nickname}: other message: {:#?}", message.message)
                }
            }
        }
        println!("* recv closed");
        anyhow::Ok(())
    });

    let global_send = tokio::task::spawn(async move {
        let mut input = BufReader::new(tokio::io::stdin()).lines();
        while let Some(line) = input.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            println!("* sending message: {line}");
            global_sender.broadcast_message(line.to_string().into()).await?;
        }
        println!("* sender closed.");
        anyhow::Ok(())
    });


    // /////////////////////////
    // 
    let server_chat = server_join_server_chat(global_mm.clone()).await?;
    let server_sender = server_chat.sender();
    server_sender.set_presence(&ServerChatPresence{
        is_server:true,
    }).await;
    let server_receiver = server_chat.receiver().await;
    
    let server_recv_thread = tokio::task::spawn(async move {

        while let Some(message) = server_receiver.next_message().await {
            let (to, message) = 
                server_process_message(message).await;
            let _r = server_sender.direct_message(to, message).await;
            if _r.is_err() {
                tracing::error!("erroR: {_r:#?}");
            }
        }
        Ok(())
    });


    // /////////////////////////
    //    INIT

    let _r1 =  n0_future::future::race(n0_future::future::race(global_receive, global_send), server_recv_thread).await?;

    info!("* SERVER server_main_loop closed.");

    Ok(())
}


async fn server_join_server_chat(mm: GlobalMatchmaker) -> anyhow::Result<ChatController<ServerChatRoomType>> {
    let Some(nn) = mm.own_node().await else {
        anyhow::bail!("server_join_server_chat: no node!");
    };
    let chat_ticket =
        ChatTicket::new_str_bs("server", BTreeSet::from([]));
    let Ok(chat) = nn.join_chat::<ServerChatRoomType>(&chat_ticket).await
    else {
        anyhow::bail!("server_join_server_chat: Failed to join server chat");
    };
    Ok(chat)
}

async fn server_process_message(message: ReceivedMessage<ServerChatRoomType>) -> (NodeIdentity, ServerChatMessageContent) {
    
    let from = message.from;
    let message = message.message;
    let ServerChatMessageContent::Request(request) =  message else {
        return (from, ServerChatMessageContent::Reply(Err("message not request!".to_string())))
    };

    let reply = server_compute_reply(from, request).await
    .map_err(move |e| e.to_string());


    (from, ServerChatMessageContent::Reply(reply))
}


async fn server_compute_reply(from: NodeIdentity, request: ServerMessageRequest) -> anyhow::Result<ServerMessageReply> {
    Ok(match request {
        ServerMessageRequest::GuestLoginMessage {  } => {
            db_add_guest_login(from).await?;
            ServerMessageReply::GuestLoginMessage {  }
        },
    })
}
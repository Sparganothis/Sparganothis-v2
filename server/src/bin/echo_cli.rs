use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use protocol::{
    chat::{ChatMessage, NetworkChangeEvent, NetworkEvent}, global_matchmaker::GlobalMatchmaker, user_identity::UserIdentitySecrets,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::StreamExt;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let id = UserIdentitySecrets::generate();
    let global_mm = GlobalMatchmaker::new(Arc::new(id)).await?;

    let _mm = global_mm.clone();
    let _a = tokio::spawn(async move {
        cli_chat_window(_mm).await.unwrap();
    });

    tokio::signal::ctrl_c().await?;
    _a.abort();

    global_mm.shutdown().await?;

    std::process::exit(0);

    // Ok(())
}

async fn cli_chat_window(global_mm: GlobalMatchmaker) -> Result<()> {
    // let mut our_ticket = ticket.clone();
    // our_ticket.bootstrap = [node.node_id()].into_iter().collect();
    // println!("* ticket to join this chat:");
    // println!("{}", our_ticket.serialize());

    println!("* waiting for peers ...");
    let controller = global_mm.global_chat_controller().await.unwrap();
    let sender = controller.sender();
    let mut receiver = controller.receiver();

    println!("* join OK");

    let receive = tokio::task::spawn(async move {
        while let Some(event) = receiver.next().await {
            let event = event?;
            match event {
                NetworkEvent::NetworkChange { event } =>  match event {
                    NetworkChangeEvent::Joined { neighbors } => {
                        println!("* swarm joined {} neighbors", neighbors.len());
                    }
                    NetworkChangeEvent::NeighborUp { node_id } => {
                        println!("* neighbor up: {}", node_id.fmt_short());
                    }
                    NetworkChangeEvent::NeighborDown { node_id } => {
                        println!("* neighbor down: {}", node_id.fmt_short());
                    }
                    NetworkChangeEvent::Lagged => {
                        println!("* lagged");
                    }
                },
                NetworkEvent::Message {
                    event
                } => match event.message {
                    ChatMessage::Message { text } => {
                        let nickname = event.from.nickname();
                        let node_id = event.from.node_id().fmt_short();
                        let user_id = event.from.user_id().fmt_short();
                        
                        println!("<{user_id}@{node_id}> {nickname}: {text}");
                    },
                    _ => ()
                }
            }
        }
        println!("* closed");
        anyhow::Ok(())
    });

    let send = tokio::task::spawn(async move {
        let mut input = BufReader::new(tokio::io::stdin()).lines();
        while let Some(line) = input.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            println!("* sending message: {line}");
            sender.send(line.to_string()).await?;
        }
        println!("* sender closed.");
        anyhow::Ok(())
    });

    // TODO: Clean shutown.
    receive.await??;
    // println!("* receive closed.");
    send.await??;

    info!("* CLI chat closed.");

    Ok(())
}

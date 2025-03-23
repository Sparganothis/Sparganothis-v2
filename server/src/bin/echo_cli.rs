use std::sync::Arc;

use anyhow::Result;
use protocol::{
    chat::{ChatMessage, NetworkChangeEvent, NetworkEvent},
    global_matchmaker::{GlobalChatPresence, GlobalMatchmaker},
    user_identity::UserIdentitySecrets,
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

    let _r = n0_future::future::race(
        async move {
            let _r = cli_chat_window(_mm).await;
            println!("* cli_chat_window closed: {:?}", _r);
        },
        async move {
            let _r = tokio::signal::ctrl_c().await;
            println!("* ctrl-c received");
        },
    )
    .await;

    global_mm.shutdown().await?;

    println!("* shutdown OK");
    // std::process::exit(0);

    Ok(())
}

async fn cli_chat_window(global_mm: GlobalMatchmaker) -> Result<()> {
    // let mut our_ticket = ticket.clone();
    // our_ticket.bootstrap = [node.node_id()].into_iter().collect();
    // println!("* ticket to join this chat:");
    // println!("{}", our_ticket.serialize());

    println!("* waiting for peers ...");
    let controller = global_mm.global_chat_controller().await.unwrap();
    controller.set_presence(&GlobalChatPresence {
        url: "".to_string(),
        platform: "CLI".to_string(),
    }).await;
    let sender = controller.sender();
    let mut receiver = controller.receiver();

    println!("* join OK");

    let receive = tokio::task::spawn(async move {
        while let Some(event) = receiver.next().await {
            let event = event?;
            match event {
                NetworkEvent::NetworkChange { event } => match event {
                    NetworkChangeEvent::Joined { neighbors } => {
                        println!(
                            "* swarm joined {} neighbors",
                            neighbors.len()
                        );
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
                NetworkEvent::Message { event } => match event.message {
                    ChatMessage::Message { text } => {
                        let nickname = event.from.nickname();
                        let node_id = event.from.node_id().fmt_short();
                        let user_id = event.from.user_id().fmt_short();

                        println!("<{user_id}@{node_id}> {nickname}: {text}");
                    }
                    _ => (),
                },
            }
        }
        println!("* recv closed");
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

    let _r = n0_future::future::race(receive, send).await?;

    info!("* CLI chat closed.");

    Ok(())
}

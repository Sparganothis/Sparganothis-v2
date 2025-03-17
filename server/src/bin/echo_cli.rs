use std::collections::HashMap;

use anyhow::Result;
use iroh::SecretKey;
use protocol::{chat::{ChatTicket, ChatEvent, TopicId}, global_matchmaker::{GlobalChatController, GlobalMatchmaker}};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::StreamExt;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let random_key = SecretKey::generate(rand::thread_rng());
    let global_mm = GlobalMatchmaker::new(random_key).await?;


    let _mm = global_mm.clone();
    let _a = tokio::spawn(async move {
        cli_chat_window(_mm).await.unwrap();
    });

    tokio::signal::ctrl_c().await?;
    _a.abort();

    global_mm.shutdown().await?;

    // let node = EchoNode::spawn().await?;
    //      Command::Connect { node_id, payload } => {
    //         let mut events = node.connect(node_id, payload);
    //         while let Some(event) = events.next().await {
    //             println!("event {event:?}");
    //         }
    //     }
    //     Command::Accept => {
    //         println!("connect to this node:");
    //         println!(
    //             "cargo run -- connect {} {}",
    //             node.endpoint().node_id(),
    //             "hello-please-echo-back"
    //         );
    //         let mut events = node.accept_events();
    //         while let Some(event) = events.next().await {
    //             println!("event {event:?}");
    //         }
    //     }
    println!("* main closed.");

    std::process::exit(0);

    // Ok(())
}


async fn cli_chat_window(global_mm: GlobalMatchmaker) -> Result<()> {

    // let mut our_ticket = ticket.clone();
    // our_ticket.bootstrap = [node.node_id()].into_iter().collect();
    // println!("* ticket to join this chat:");
    // println!("{}", our_ticket.serialize());

    println!("* waiting for peers ...");
    let mut controllers = global_mm.take_global_chat_controllers().await;
    let GlobalChatController { sender, mut receiver } = controllers.take().unwrap();

    println!("* join OK");

    let receive = tokio::task::spawn(async move {
        let mut names = HashMap::new();
        while let Some(event) = receiver.try_next().await? {
            match event {
                ChatEvent::Joined { neighbors } => {
                    println!("* swarm joined");
                    for node_id in neighbors {
                        println!("* neighbor up: {node_id}")
                    }
                }
                ChatEvent::Presence {
                    from,
                    nickname,
                    sent_timestamp: _,
                } => {
                    let from_short = from.fmt_short();
                    if !nickname.is_empty() {
                        let old_name = names.get(&from);
                        if old_name != Some(&nickname) {
                            println!("* {from_short} is now known as {nickname}")
                        }
                    }
                    names.insert(from, nickname.clone());
                }
                ChatEvent::MessageReceived {
                    from,
                    text,
                    nickname,
                    sent_timestamp: _,
                } => {
                    let from_short = from.fmt_short();
                    if !nickname.is_empty() {
                        let old_name = names.get(&from);
                        if old_name != Some(&nickname) {
                            println!("* {from_short} is now known as {nickname}")
                        }
                    }
                    println!("<{from_short}> {nickname}: {text}");
                }
                ChatEvent::NeighborUp { node_id } => {
                    println!("* neighbor up: {node_id}")
                }
                ChatEvent::NeighborDown { node_id } => {
                    println!("* neighbor down: {node_id}")
                }
                ChatEvent::Lagged => {
                    println!("* warn: gossip stream lagged")
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

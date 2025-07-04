use std::sync::Arc;

use anyhow::Result;
use protocol::{
    chat::{IChatController, IChatReceiver, IChatSender},
    global_chat::{GlobalChatMessageContent, GlobalChatPresence},
    global_matchmaker::GlobalMatchmaker,
    user_identity::UserIdentitySecrets,
};
use server::server::main_loop::server_main_loop;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let id = UserIdentitySecrets::generate();
    let global_mm = GlobalMatchmaker::new(Arc::new(id)).await?;

    let _mm = global_mm.clone();

    let _r = n0_future::future::race(
        async move {
            let _r = server_main_loop(_mm).await;
            println!("* server_main_loop closed: {:?}", _r);
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

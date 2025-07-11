use std::collections::HashMap;

use n0_future::{FuturesUnordered, StreamExt};
use rand::{thread_rng, Rng};
use server::server::multiplayer::matchmaker::run_multiplayer_matchmaker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    {
        let sub = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO);

        tracing::subscriber::set_global_default(sub.finish()).unwrap();
    }

    main_run().await?;

    // wait_for_killer_to_die().await;

    Ok(())
}

async fn main_run() -> anyhow::Result<()> {
    const PLAYERS_PER_SECOND: f32 = 7.0;
    const PLAYER_COUNT: usize = 400;
    const SLEEP_S: f64 = 1.0 / PLAYERS_PER_SECOND as f64;

    let mut fut = FuturesUnordered::new();
    for i in 0..PLAYER_COUNT {
        fut.push(async move {
            tokio::time::sleep(std::time::Duration::from_secs_f64(
                SLEEP_S * (i + 1) as f64,
            ))
            .await;

            let random: u128 = (&mut thread_rng()).gen();
            let random = format!("{}", random)[0..6].to_string();

            (
                i,
                random.clone(),
                run_multiplayer_matchmaker(random.clone(), "1v1", 2).await,
            )
        });
    }
    let mut ok_count = 0;
    let mut err_count = 0;
    let mut result_list = vec![];
    while let Some((i, random, n)) = fut.next().await {
        match n {
            Ok(_list) => {
                tracing::info!(
                    "player {i} SUCCESS IN MATCHMAKING. ID LIST = {_list:?}"
                );
                ok_count += 1;
                result_list.push((i, random, _list));
            }
            Err(e) => {
                tracing::error!("player {i} FAILED MATCHMAKING: {e}.");
                err_count += 1;
            }
        }
    }
    tracing::info!(
        "ERR COUNT = {}, OK COUNT = {}, TOTAL COUNT = {}",
        err_count,
        ok_count,
        err_count + ok_count
    );
    result_list.sort_by_key(|f| f.2.clone());
    for (i, random, list) in result_list.clone() {
        tracing::info!("OK i={i:04} random={random} list={list:?}");
    }
    let mut result_count = HashMap::new();
    for l in result_list {
        let entry = result_count.entry(format!("{:?}", l.2)).or_insert(0);
        *entry += 1;
    }
    for (pair, count) in result_count.iter() {
        if *count != 2 {
            tracing::error!("NOT EXACTLY 2: {pair} = {count}");
        }
    }
    Ok(())
}

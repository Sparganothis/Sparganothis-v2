use std::collections::HashMap;

use game::api::game_match::GameMatchType;
use n0_future::{FuturesUnordered, StreamExt};
use protocol::user_identity::{self, NodeIdentity, UserIdentitySecrets};
use rand::{thread_rng, Rng};
use server::server::multiplayer::matchmaker::matchmaker_api::{
    run_multiplayer_matchmaker_1, run_multiplayer_matchmaker_2,
};

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
    const PLAYERS_PER_SECOND: f32 = 14.0;
    const PLAYER_COUNT: usize = 20;
    const SLEEP_S: f64 = 1.0 / PLAYERS_PER_SECOND as f64;

    let mut fut = FuturesUnordered::new();
    for i in 0..PLAYER_COUNT {
        fut.push(async move {
            tokio::time::sleep(std::time::Duration::from_secs_f64(
                SLEEP_S * (i + 1) as f64,
            ))
            .await;

            let user_secrets = UserIdentitySecrets::generate();
            let node_identity = NodeIdentity::new(
                *user_secrets.user_identity(),
                user_secrets.user_identity().user_id().clone(),
                None,
            );

            let r = run_multiplayer_matchmaker_1(
                node_identity,
                game::api::game_match::GameMatchType::_1v1,
            )
            .await;
            let mut r2 = None;
            if let Ok(v) = r {
                let t3 = run_multiplayer_matchmaker_2(
                    node_identity,
                    (GameMatchType::_1v1, v),
                )
                .await;
                if let Ok(t3) = t3 {
                    r2 = Some(t3);
                }
            }
            let rf = match r2 {
                Some(x) => Ok(x),
                None => Err(anyhow::anyhow!("fail.")),
            };
            (i, node_identity.clone(), rf)
        });
    }
    let mut ok_count = 0;
    let mut err_count = 0;
    let mut result_list = vec![];
    while let Some((i, _node_id, n)) = fut.next().await {
        match n {
            Ok(_list) => {
                let _nickname = _node_id.nickname();
                let _matchid = _list.match_id;

                tracing::info!(
                    "player {i} {_nickname} SUCCESS IN MATCHMAKING. = {_matchid}"
                );
                ok_count += 1;
                result_list.push((_list.match_id, i));
            }
            Err(e) => {
                tracing::error!("player {i} FAILED MATCHMAKING: {e}.");
                err_count += 1;
            }
        }
    }
    result_list.sort();
    tracing::info!(
        "ERR COUNT = {}, OK COUNT = {}, TOTAL COUNT = {}",
        err_count,
        ok_count,
        err_count + ok_count
    );
    let mut result_count = HashMap::new();
    for l in result_list {
        let entry = result_count.entry(format!("{:?}", l.0)).or_insert(0);
        *entry += 1;
    }
    for (pair, count) in result_count.iter() {
        if *count != 2 {
            tracing::error!("NOT EXACTLY 2: {pair} = {count}");
        }
    }
    Ok(())
}

use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Context;
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use game::timestamp::get_timestamp_now_ms;
use rand::{thread_rng, Rng};
use redis::Value;
use tokio::{
    time::{sleep, timeout},
};
use tracing::info;

const REDIS_CLIENT: std::cell::OnceCell<redis::Client> =
    std::cell::OnceCell::new();

async fn redis_connection() -> anyhow::Result<redis::aio::MultiplexedConnection>
{
    let client = REDIS_CLIENT
        .get_or_init(move || {
            redis::Client::open("redis://127.0.0.1:6379").unwrap()
        })
        .clone();
    Ok(client.get_multiplexed_tokio_connection().await?)
}

// #[derive(Clone)]
// struct Killer {
//     tx: tokio::sync::mpsc::UnboundedSender<Option<String>>,
//     _handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
// }

// fn make_killer() -> Killer {
//     tracing::info!("SUPER DUPER CONSTRUCTOR>");
//     let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
//     let h = tokio::task::spawn(async move {
//         tracing::info!("START KILLER");
//         while let Some(Some(r)) = rx.recv().await {
//             tracing::info!(" KILLER MESSAGE = {r}");
//             if let Err(e) = drop_lock(r).await {
//                 tracing::error!("cannot drop redis lock: {e:#?}");
//             }
//         }
//         info!("KILLER EXIT!");
//     });
//     Killer {
//         tx,
//         _handle: Arc::new(Mutex::new(Some(h))),
//     }
// }

// lazy_static::lazy_static! {
//     static ref UNLOCK_KILLER: Killer = {
//         make_killer()
//     };
// }

// fn get_killer() -> &'static Killer {
//     &UNLOCK_KILLER
// }

// fn send_to_kill(key: String) {
//     let key2 = key.clone();
//     let killer = get_killer();
//     if let Err(e) = killer.tx.send(Some(key2)) {
//         tracing::error!("cannot send to kill list: {:#?}", e);
//     }
// }

// fn kill_killer() {
//     let killer = get_killer();
//     if let Err(e) = killer.tx.send(None) {
//         tracing::error!("cannot send to kill list: {:#?}", e);
//     }
// }

// async fn wait_for_killer_to_die() {
//     info!("WAIT FOR KILLER TO DIE!");
//     kill_killer();
//     tokio::time::sleep(Duration::from_millis(20)).await;
//     let killer = get_killer();
//     let _x = { killer._handle.lock().await.take() };
//     if let Some(_x) = _x {
//         let _x = _x.await;
//     }
// }

pub struct LockGuard {
    _key: String,
}
// impl Drop for LockGuard {
//     fn drop(&mut self) {
//         // if ts > self.expires_at {
//         //     return;
//         // }
//         info!("spawn drop lock guard XXXXXXX");
//         let key = self.key.clone();
//         send_to_kill(key);
//     }
// }

// returns true if locked, false if not locked (already locked), error on error.
async fn set_lock(
    key: &str,
    val: &str,
    ttl_ms: i64,
) -> anyhow::Result<Option<LockGuard>> {
    let mut conn = redis_connection().await?;
    let _r = redis::cmd("SET")
        .arg(key)
        .arg(val)
        .arg("nx")
        .arg("px")
        .arg(ttl_ms)
        .query_async::<Value>(&mut conn)
        .await?;
    info!("_r: {:#?}", _r);
    if matches!(_r, Value::Okay) {
        return Ok(Some(LockGuard {
            _key: key.to_string(),
        }));
    };
    Ok(None)
}

async fn get_lock_value(key: &str) -> anyhow::Result<Option<String>> {
    let mut conn = redis_connection().await?;
    let _r = redis::cmd("GET")
        .arg(key)
        .query_async::<String>(&mut conn)
        .await;

    let _r = _r.ok();
    Ok(_r)
}

// true if deleted, false if not deleted
async fn drop_lock(key: String) -> anyhow::Result<bool> {
    info!("DROPPING LOCK");
    let mut conn = redis_connection().await?;
    let _r = redis::cmd("DEL")
        .arg(key)
        .query_async::<Value>(&mut conn)
        .await;
    let _r = _r.is_ok();
    Ok(_r)
}

struct MatchmakingBroadcaster {
    tx: async_broadcast::Sender<BroadcastedItem>,
    rx: async_broadcast::InactiveReceiver<BroadcastedItem>,
}

#[derive(Clone)]
struct BroadcastedItem {
    vals: Arc<Vec<String>>,
}

fn make_broadcaster() -> MatchmakingBroadcaster {
    let (mut tx, mut rx) = async_broadcast::broadcast(1);
    tx.set_overflow(true);
    rx.set_overflow(true);
    MatchmakingBroadcaster {
        tx,
        rx: rx.deactivate(),
    }
}

lazy_static::lazy_static! {
    static ref MATCHMAKING_SUCCESS_BROADCAST: MatchmakingBroadcaster = {
        make_broadcaster()
    };
}

async fn get_lock_for_match(
    match_user_count: i32,
    match_type: String,
    timeout_ms: i32,
    user_id_value: String,
) -> anyhow::Result<(Vec<String>, LockGuard)> {
    let mut lock_keys = vec![];
    let mut lock_values = HashMap::new();
    let mut possible_insert_keys = vec![];

    for i in 1..=match_user_count {
        let key = format!("{match_type}_user_{i}");
        let val = get_lock_value(&key).await?;
        if val.is_none() {
            possible_insert_keys.push(key.clone());
        }
        lock_values.insert(key.clone(), val);
        lock_keys.push(key);
    }
    if possible_insert_keys.is_empty() {
        anyhow::bail!("full queue for this key. try again later.");
    }

    let mut event_rx = MATCHMAKING_SUCCESS_BROADCAST.rx.activate_cloned();
    let idx =
        (&mut rand::thread_rng()).gen_range(0..{ possible_insert_keys.len() });
    let insert_key = possible_insert_keys[idx].clone();

    let _lock =
        set_lock(&insert_key, &user_id_value, timeout_ms as i64).await?;
    let Some(_lock) = _lock else {
        anyhow::bail!("someone else got my lock!");
    };
    lock_values.insert(insert_key.clone(), Some(user_id_value.clone()));
    let user_id_value2 = user_id_value.clone();
    let fut_we_found_group = async move {
        const RETRY_COUNT: u64 = 16;
        for _retry in 0..RETRY_COUNT - 1 {
            tokio::time::sleep(Duration::from_millis(
                timeout_ms as u64 / RETRY_COUNT,
            ))
            .await;
            for key in lock_keys.iter() {
                let val = get_lock_value(&key).await?;
                lock_values.insert(key.clone(), val);
            }
            // check if all are in
            let all_in = lock_values.values().all(Option::is_some);
            if all_in {
                let mut values = lock_values
                    .values()
                    .filter_map(|x| x.clone())
                    .collect::<Vec<_>>();
                values.sort();
                if !values.contains(&user_id_value2) {
                    anyhow::bail!(
                        "we dont find ourselves in latest list = abort"
                    )
                };
                let _x = MATCHMAKING_SUCCESS_BROADCAST
                    .tx
                    .broadcast(BroadcastedItem {
                        vals: values.clone().into(),
                    })
                    .await?;
                for x in lock_keys.iter() {
                    match drop_lock(x.clone()).await {
                        Ok(_x) => {
                            tracing::info!("DROP LOCK {x} OK {_x}");
                        }
                        Err(e) => {
                            tracing::error!("DROP LOCK {x} ERR {e}");
                        }
                    }
                }
                tracing::warn!("MATCH! fut_we_found_group:  for  userid={user_id_value2} list={values:?}");
                return Ok(values);
            }
        }

        anyhow::bail!("timed out {timeout_ms} no complete match making!")
    };
    let fut_we_found_group = async move {
        anyhow::Ok(
            tokio::time::timeout(
                Duration::from_millis((timeout_ms as f64 * 0.9) as u64),
                fut_we_found_group,
            )
            .await??,
        )
    };

    let user_id_value2 = user_id_value.clone();
    let fut_someone_else_found_group = async move {
        while let Some(msg) = event_rx.next().await {
            let v = &msg.vals;
            if v.contains(&user_id_value2) {
                let v2: Vec<_> = v.iter().cloned().collect();
                tracing::warn!("MATCH! fut_someone_else_found_group:  for  userid={user_id_value2} list={v2:?}");
                return Ok(v2);
            }
        }
        anyhow::bail!("EXIT fut_someone_else_found_group")
    };
    let fut_someone_else_found_group = async move {
        anyhow::Ok(
            tokio::time::timeout(
                Duration::from_millis((timeout_ms as f64 * 1.5) as u64),
                fut_someone_else_found_group,
            )
            .await??,
        )
    };

    let mut fut = FuturesUnordered::new();
    fut.push(fut_someone_else_found_group.boxed());
    fut.push(fut_we_found_group.boxed());

    let f1 = fut.next().await.context("fut")?;

    let f2 = fut.next().await.context("fut")?;

    let f3 = f1.or(f2);

    let f4 = f3.map(|v| (v, _lock));
    f4

    // let final_value = n0_future::future::race(fut_someone_else_found_group, fut_we_found_group).await;

    // let we_found = fut_we_found_group.await;
    // if let Ok(our_finding) = we_found {
    //     return Ok((our_finding, _lock))
    // }

    // let others_found = fut_someone_else_found_group.await?;
    // Ok((others_found, _lock))
}

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
    const PLAYERS_PER_SECOND: usize = 1;
    const PLAYER_COUNT: usize = 10;
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
                player_matchmaking_run(random.clone()).await,
            )
        });
    }
    let mut ok_count = 0;
    let mut err_count = 0;
    let mut result_list = vec![];
    while let Some((i, random, n)) = fut.next().await {
        match n {
            Ok((_list, _lock_guard)) => {
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
    for (i, random, list) in result_list {
        tracing::info!("OK i={i:04} random={random} list={list:?}");
    }
    Ok(())
}

async fn player_matchmaking_run(
    random: String,
) -> anyhow::Result<(Vec<String>, LockGuard)> {
    const MATCHMAKING_TIMEOUT: i64 = 30000;
    const TOTAL_RETRY_INTERVAL_MS: i64 = 3333;
    const ITERATION_TIMEOUT_MS: i32 = 1000;
    for _i in 0..(MATCHMAKING_TIMEOUT / TOTAL_RETRY_INTERVAL_MS) {
        let _p = player_matchmaking_run1(
            random.clone(),
            TOTAL_RETRY_INTERVAL_MS,
            ITERATION_TIMEOUT_MS,
        )
        .await;
        if let Ok(r) = _p {
            return Ok(r);
        }

        let t0 = get_timestamp_now_ms();
        let tnext =
            t0 - (t0 % TOTAL_RETRY_INTERVAL_MS) + TOTAL_RETRY_INTERVAL_MS;
        let diff_ms = tnext - t0;
        if diff_ms > 0 {
            sleep(Duration::from_millis(diff_ms as u64)).await;
        }
    }
    anyhow::bail!("matchmaking timouet")
}

async fn player_matchmaking_run1(
    random: String,
    timeout_msz: i64,
    iter_timeout_ms: i32,
) -> anyhow::Result<(Vec<String>, LockGuard)> {
    let fut = async move {
        let t0 = get_timestamp_now_ms();
        while get_timestamp_now_ms() - t0 < timeout_msz {
            let _r = get_lock_for_match(
                2,
                "1v1".to_string(),
                iter_timeout_ms,
                random.clone(),
            )
            .await;
            if let Ok(_r) = _r {
                return Ok(_r);
            }

            let random_sleep: i32 = (&mut rand::thread_rng())
                .gen_range(iter_timeout_ms / 20..iter_timeout_ms / 10);

            sleep(Duration::from_millis(random_sleep as u64)).await;
        }
        anyhow::bail!("player matchmaking timeout.");
    };

    let fut = timeout(Duration::from_millis(timeout_msz as u64), fut).await??;
    Ok(fut)
}

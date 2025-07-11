use std::{cell::OnceCell, collections::{BTreeSet, HashMap, HashSet}, sync::Arc, time::Duration};

use anyhow::Context;
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use game::timestamp::get_timestamp_now_ms;
use rand::{thread_rng, Rng};
use redis::Value;
use tokio::{
    sync::Mutex,
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::{info, warn};

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

struct LockGuard {
    _key: String,
}

// returns true if locked, false if not locked (already locked), error on error.
async fn set_lock(
    key: &str,
    val: &str,
    ttl_ms: i64,
) -> anyhow::Result<LockGuard> {
    let t0 = get_timestamp_now_ms();
    let mut conn = redis_connection().await?;
    let _r = redis::cmd("SET")
        .arg(key)
        .arg(val)
        .arg("nx")
        .arg("px")
        .arg(ttl_ms)
        .query_async::<Value>(&mut conn)
        .await?;
    if matches!(_r, Value::Okay) {
        info!("SET LOCK {key} = {val} dt={}ms", get_timestamp_now_ms()-t0);
        return Ok(LockGuard {
            _key: key.to_string(),
        });
    };
    anyhow::bail!("lock fail.")
}
/// Takes one username, game type and match user count - and returns a sorted list of the usernames of all the players in the match (if matchmaking succeeded)
pub async fn run_multiplayer_matchmaker(
    username_val: String,
    game_type: &str,
    match_user_count: u8,
) -> anyhow::Result<Vec<String>> {
    const MATCHMAKING_TIMEOUT: i64 = 30000;
    const TOTAL_RETRY_INTERVAL_MS: i64 = 3333;
    const ITERATION_TIMEOUT_MS: i32 = 1000;
    for _i in 0..(MATCHMAKING_TIMEOUT / TOTAL_RETRY_INTERVAL_MS) {
        let t0 = get_timestamp_now_ms();
        let tnext =
            t0 - (t0 % TOTAL_RETRY_INTERVAL_MS) + TOTAL_RETRY_INTERVAL_MS;
        let diff_ms = tnext - t0;
        if diff_ms > 0 {
            sleep(Duration::from_millis(diff_ms as u64)).await;
        }

        let _p = player_matchmaking_run1(
            username_val.clone(),
            // TOTAL_RETRY_INTERVAL_MS,
            ITERATION_TIMEOUT_MS,
            game_type,
            match_user_count,
        )
        .await;
        if let Ok(r) = _p {
            return Ok(r);
        }
    }
    anyhow::bail!("matchmaking timouet")
}

async fn player_matchmaking_run1(
    user_id: String,
    // round_time: i64,
    fetch_time: i32,
    game_type: &str,
    match_user_count: u8,
) -> anyhow::Result<Vec<String>> {
    let t0 = get_timestamp_now_ms();
    let player_lot = fetch_player_slot(
        user_id,
        fetch_time,
        &game_type,
        fetch_time as i64 * 2,
    );
    let player_lot =
        timeout(Duration::from_millis(fetch_time as u64), player_lot).await??;
    let t1 = get_timestamp_now_ms();
    let dt = t1 - t0;
    if dt < fetch_time as i64 {
        let slp = fetch_time - dt as i32;
        if slp > 0 {
            sleep(Duration::from_millis(slp as u64)).await;
        }
    }
    // we are now sync again!
    sleep(Duration::from_millis(fetch_time as u64/20)).await;

    let mut interesting_ids = vec![];
    let root_id = player_lot.0 - player_lot.0 % match_user_count as i32;
    for i in root_id..(root_id + match_user_count as i32) {
        interesting_ids.push(make_key(&game_type, i))
    }

    let read_retry_count = 3;
    for _r in 0..read_retry_count {
        let v = get_lock_values(interesting_ids.clone()).await?;
        if v.len() == interesting_ids.len() {
            let mut v = v;
            v.sort();
            return Ok(v)
        }
        sleep(Duration::from_millis(fetch_time as u64 / (read_retry_count  as u64 + 1))).await;
    }
    anyhow::bail!("timeout2");
}



async fn get_lock_values(keys: Vec<String>) -> anyhow::Result<Vec<String>> {
    let t0 = get_timestamp_now_ms();
    let mut conn = redis_connection().await?;
    let _r = redis::cmd("MGET")
        .arg(keys.clone())
        .query_async::<Vec<Option<String>>>(&mut conn)
        .await;

    let _r: Vec<String> = _r?.iter().filter_map(|x| x.clone()).collect();
    let dt = get_timestamp_now_ms() - t0;
    info!("MGET {} keys -> {} vals took {}ms", keys.len(), _r.len(), dt);
    Ok(_r)
}



fn make_key(game_type: &str, i: i32) -> String {
    format!("mm_lock_{game_type}_{i}")
}

async fn fetch_player_slot(
    user_id: String,
    _fetch_time: i32,
    game_type: &str,
    ttl_ms: i64,
) -> anyhow::Result<(i32, String)> {
    let t0 = get_timestamp_now_ms();
    let _rand_sleep2: i32 
        = (&mut thread_rng()).gen_range(0.._fetch_time/5);
    sleep(Duration::from_millis(_rand_sleep2 as u64)).await;

    let mut all_keys = vec![];
    let mut key_to_i = HashMap::new();
    for i in 0..100 {
        let key = make_key(&game_type, i);
        all_keys.push(key.clone());
        key_to_i.insert(key, i);
    }

    let already_present_keys = get_lock_values(all_keys.clone()).await?;
    let all_keys = BTreeSet::from_iter(all_keys.iter().cloned());
    let already_present_keys = BTreeSet::from_iter(already_present_keys.iter().cloned());
    let mut all_keys = Vec::from_iter(all_keys.iter().cloned());
    all_keys.sort_by_key(|x| key_to_i[x]);

    for key in all_keys {
        if already_present_keys.contains(&key) {
            continue;
        }
        // we can check this lock !

        if let Ok(_l) = set_lock(&key, &user_id, ttl_ms).await {
            return Ok((key_to_i[&key], key));
        }
        if get_timestamp_now_ms() - t0 > _fetch_time as i64 {
            break;
        }
    }

    anyhow::bail!("no more keys to check!");
}


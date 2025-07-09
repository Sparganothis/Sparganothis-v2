use std::{cell::OnceCell, sync::Arc, time::Duration};

use game::timestamp::get_timestamp_now_ms;
use rand::{thread_rng, Rng};
use redis::Value;
use server::server::redis::lock::RedLock;
use tokio::sync::Mutex;
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

#[derive(Clone)]
struct Killer {
    tx: tokio::sync::mpsc::UnboundedSender<Option<String>>,
    _handle: Arc::<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}


const UNLOCK_KILLER: OnceCell<Killer> =
    OnceCell::new();


fn get_killer() -> Killer {
let killer = UNLOCK_KILLER
        .get_or_init(move || {
            tracing::info!("SUPER DUPER CONSTRUCTOR>");
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let h = tokio::task::spawn(async move {
                tracing::info!("START KILLER");
                while let Some(Some(r)) = rx.recv().await {
                    tracing::info!(" KILLER MESSAGE = {r}");
                    if let Err(e) = drop_lock(r).await {
                        tracing::error!("cannot drop redis lock: {e:#?}");
                    }
                }
                info!("KILLER EXIT!");
            });
            Killer{
                tx, _handle: Arc::new(Mutex::new(Some(h))),
            }
        })
        .clone();
    killer
}


fn send_to_kill(key: String) {
    let key2 = key.clone();
    let killer = get_killer();
    if let Err(e) = killer.tx.send(Some(key2)) {
        tracing::error!("cannot send to kill list: {:#?}", e);
    }
}

async fn wait_for_killer_to_die() {
    info!("WAIT FOR KILLER TO DIE!");
    tokio::time::sleep(Duration::from_millis(20)).await;
    if let Some(killer) = UNLOCK_KILLER.get() {
        let _x =  {killer._handle.lock().await.take()};
        if let Some(_x) = _x {
            let _x = _x.await;
        }
    } else {
        warn!("NO KILLER REGISTERED");
    }
}

pub struct LockGuard {
    key: String,
}
impl Drop for LockGuard {
    fn drop(&mut self) {
        // if ts > self.expires_at {
        //     return;
        // }
        info!("spawn drop lock guard XXXXXXX");
        let key = self.key.clone();
        send_to_kill(key);
    }
}

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
            key: key.to_string(),
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    {
        let sub = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO);

        tracing::subscriber::set_global_default(sub.finish()).unwrap();
    }
    let random: u128 = (&mut thread_rng()).gen();
    let random = format!("{}", random);

    match set_lock("gigi", &random, 3000).await {
        Ok(Some(_lock)) => {
            info!("WE HAVE THE LOCK!");
            let v = get_lock_value("gigi").await?;
            println!("LOCK BEFORE SLEEP: {v:?}");
            tokio::time::sleep(Duration::from_millis(2000)).await;
            let v = get_lock_value("gigi").await?;
            println!("LOCK AFTER SLEEP: {v:?}");
        }
        Ok(None) => {
            info!("WE DO NOT HAVE THE LOCK!");
        }
        Err(e) => {
            tracing::error!("ERROR GETTING LOCK: {e:#?}");
        }
    }

    wait_for_killer_to_die().await;


    Ok(())
}

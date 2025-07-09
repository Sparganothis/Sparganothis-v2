use redis::Value;
use server::server::redis::lock::RedLock;
use tracing::info;


const REDIS_CLIENT : std::cell::OnceCell<redis::Client> = std::cell::OnceCell::new();

async fn redis_connection() -> anyhow::Result<redis::aio::MultiplexedConnection> {
    let client = REDIS_CLIENT.get_or_init(move || {
            redis::Client::open("redis://127.0.0.1:6379").unwrap()
    }).clone();
    Ok(client.get_multiplexed_tokio_connection().await?)
}



// returns true if locked, false if not locked (already locked), error on error.
async fn set_lock(key: &str, val: &str, ttl_ms: i32)->anyhow::Result<bool> {
    let mut conn = redis_connection().await?;
    let _r = redis::cmd("SET").arg(key).arg(val).arg("nx").arg("px").arg(ttl_ms).exec_async(&mut conn).await;
    
    Ok(_r.is_ok())
}


async fn get_lock_value(key: &str) -> anyhow::Result<Option<String>> {
    let mut conn = redis_connection().await?;
    let _r = redis::cmd("GET").arg(key).query_async::<String>(&mut conn).await;

    let _r = _r.ok();
    Ok(_r)
}

// true if deleted, false if not deleted
async fn drop_lock(key: &str) -> anyhow::Result<bool> {
    let mut conn = redis_connection().await?;
    let _r = redis::cmd("DEL").arg(key).query_async::<Value>(&mut conn).await;
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

    match set_lock("gigi", "becali", 50000).await {
        Ok(true) => {
            info!("WE HAVE THE LOCK!");
        }
        Ok(false) => {
            info!("WE DO NOT HAVE THE LOCK!");
        }
        Err(e) => {
            tracing::error!("ERROR GETTING LOCK: {e:#?}");
        }
    }
   

    
    Ok(())
}


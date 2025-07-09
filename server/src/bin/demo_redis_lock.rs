use server::server::redis::lock::RedLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    {
        let sub = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO);

        tracing::subscriber::set_global_default(sub.finish()).unwrap();
    }


    let redlock_client = RedLock::new(vec!["redis://127.0.0.1:6379"]);
    let lock = redlock_client.acquire_async(b"penis", 3000).await;

    match lock {
        Ok(lock) => {
            let res = lock.lock.resource.clone()          ;
            let val = lock.lock.val.clone();
            let time_remain = lock.lock.validity_time;
            tracing::info!("res={res:?} val={val:?} time={time_remain:?}");
            for i in 0..10 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                tracing::info!("ding!");
            }
        }
        Err(err) => {
            tracing::warn!("error getting lock: {:#?}", err)
        }
    }
    
    Ok(())
}
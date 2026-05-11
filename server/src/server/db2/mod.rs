use sqlx::{Executor, MySqlPool, query};
use sqlx::mysql::MySqlPoolOptions;
use tracing::info;


static SQL_POOL: tokio::sync::OnceCell<MySqlPool> =
    tokio::sync::OnceCell::const_new();


async fn get_pool() -> sqlx::Result<&'static MySqlPool> {
    SQL_POOL
        .get_or_try_init(move || async move{
            

        info!("server SQL CONNECT...");
            MySqlPoolOptions::new().max_connections(16).connect("mariadb://root:sparganothis@127.0.0.1/sparganothis").await
        }).await
}


pub async fn init_sql() -> anyhow::Result<()> {
    let pool = get_pool().await?;

    let version_query = query!("SELECT VERSION() as version").fetch_one(pool).await?;
    info!("server SQL MARIADB VERSION = {}", version_query.version);
    
    info!("server SQL MIGRATE...");
    sqlx::migrate!("./migrations")
    .run(pool)
    .await?;

    
    info!("server SQL MIGRATE OK.");

    Ok(())
}
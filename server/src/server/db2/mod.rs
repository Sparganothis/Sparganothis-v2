use sqlx::{Executor, MySqlPool, query};
use sqlx::mysql::MySqlPoolOptions;
use tracing::info;


static SQLITE3_POOL: tokio::sync::OnceCell<MySqlPool> =
    tokio::sync::OnceCell::const_new();


async fn get_pool() -> sqlx::Result<&'static MySqlPool> {
    SQLITE3_POOL
        .get_or_try_init(move || async move{
            

        info!("server sqlite CONNECT...");
            MySqlPoolOptions::new().max_connections(16).connect("mariadb://root:tetris@127.0.0.1/tetris").await
        }).await
}


pub async fn init_sqlite_migrations() -> anyhow::Result<()> {
    return Ok(());
    // let pool = get_pool().await?;

    // let version_query = query!("SELECT sqlite_vesion()").fetch_one(&pool).await?;
    
    // info!("server sqlite MIGRATE...");
    // sqlx::migrate!("./migrations")
    // .run(pool)
    // .await?;

    
    // info!("server sqlite MIGRATE OK.");

    // Ok(())
}
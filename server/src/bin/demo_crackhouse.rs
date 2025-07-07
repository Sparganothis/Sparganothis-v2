use game::timestamp::get_timestamp_now_ms;
use server::server::db::{
    clickhouse_client::get_clickhouse_client, send_new_match::MatchRow,
};
use tracing::info;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    {
        let sub = tracing_subscriber::FmtSubscriber::builder()
            // .with_max_level(tracing::Level::INFO)
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env());

        tracing::subscriber::set_global_default(sub.finish()).unwrap();
    }

    let new_match = MatchRow {
        game_type: "test".to_string(),
        start_time: get_timestamp_now_ms(),
        user_ids: vec!["test1".to_string(), "test2".to_string()],
        game_seed: "test".to_string(),
        match_id: uuid::Uuid::new_v4().to_string(),
        data_version: 0,
        match_info: "info".to_string(),
    };

    info!("INSERT NEW MATCH!");
    let client = get_clickhouse_client();
    let mut insert = client.insert("matches")?;
    insert.write(&new_match).await?;
    insert.end().await?;

    info!("INSRT OK!");

    tracing::info!("demo crackhouse.");

    Ok(())
}

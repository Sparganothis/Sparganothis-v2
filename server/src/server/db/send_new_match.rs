use clickhouse::Row;
use game::api::game_match::GameMatch;
use protocol::{impl_api_method, server_chat_api::api_declarations::SendNewMatch, user_identity::NodeIdentity};
use serde::Serialize;
use tracing::info;

use crate::server::db::{clickhouse_client::get_clickhouse_client, guest_login::serialize_base64};

// CREATE TABLE matches
// (
//     game_type String,
//     start_time  BigInt,
//     user_ids Array(BLOB),
//     game_seed String,

//     match_info BLOB,
// )
// ENGINE = MergeTree ()
// ORDER BY (game_type, start_time, user_ids,  game_seed)
#[derive(Row, Serialize)]
pub struct MatchRow {
    pub game_type: String,
    pub start_time: i64,
    pub user_ids: Vec<String>,
    pub game_seed: String,
    pub match_id: String,
    pub data_version: i64,
    pub match_info: String,
}



pub async fn db_send_new_match(
    _from: NodeIdentity,
    (_match, ): (GameMatch<NodeIdentity>, ),
) -> anyhow::Result<()> {
    tracing::warn!("\n\n db_send_new_match !!!! \n\n !");

    if _from != _match.users[0] {
        info!("Skipping db_send_match for non-first identity");
        return anyhow::Ok(());
    }

    let game_seed = _match.seed;
    let game_seed = serialize_base64(&game_seed)?;

    
    let new_match = MatchRow {
        game_type: format!("{:?}", _match.type_),
        start_time: _match.time,
        user_ids: _match.users.iter().map(|x| serialize_base64(&x.user_id().as_bytes())).collect::<anyhow::Result<Vec<_>>>()?,
        game_seed,
        match_id: _match.match_id.clone().to_string(),
        data_version: 0,
        match_info: serialize_base64(&_match)?,
    };
    
    info!("INSERT NEW MATCH!");
    let client = get_clickhouse_client();
    let mut insert = client.insert("matches")?;
    insert.write(&new_match).await?;
    insert.end().await?;

    info!("INSRT OK!");

    Ok(())
}

impl_api_method!(SendNewMatch, db_send_new_match);

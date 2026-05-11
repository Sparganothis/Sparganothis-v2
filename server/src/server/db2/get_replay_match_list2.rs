use anyhow::Context;
use game::api::game_match::GameMatch;
use game::tet::GameState;
use protocol::api::api_declarations::GameStateRow2;
use protocol::{api::api_declarations::MatchRow2, user_identity::NodeIdentity};
use crate::server::db2::get_pool;
use crate::server::db2::guest_login::{serialize_base64, deserialize_base64};

pub async fn db_get_list_matches(
    _from: NodeIdentity,
    _arg: (),
) -> anyhow::Result<Vec<MatchRow2>> {
    tracing::info!("DB GET LIST MATCHES for user = {:?}", _from);

    let pool = get_pool().await?;
    let rows = sqlx::query!(
        r#"
SELECT game_type, start_time, user_ids, game_seed, match_id, data_version, match_info 
FROM matches
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut v = vec![];
    for row in rows {
        let user_ids: Vec<String> = deserialize_base64(row.user_ids)?;

        v.push(MatchRow2 {
            game_type: row.game_type,
            start_time: row.start_time,
            user_ids,
            game_seed: row.game_seed,
            match_id: row.match_id,
            data_version: row.data_version,
            match_info: deserialize_base64(row.match_info).ok(),
        });
    }

    Ok(v)
}

pub async fn db_get_detail_match(
    _from: NodeIdentity,
    _arg: String,
) -> anyhow::Result<MatchRow2> {
    tracing::info!("DB GET DETAIL MATCH for user = {:?}", _from);

    let pool = get_pool().await?;
    let row = sqlx::query!(
        r#"
SELECT game_type, start_time, user_ids, game_seed, match_id, data_version, match_info 
FROM matches 
WHERE match_id = ?
        "#,
        _arg
    )
    .fetch_one(pool)
    .await?;

    let user_ids: Vec<String> = deserialize_base64(row.user_ids)?;

    Ok(MatchRow2 {
        game_type: row.game_type,
        start_time: row.start_time,
        user_ids,
        game_seed: row.game_seed,
        match_id: row.match_id,
        data_version: row.data_version,
        match_info: deserialize_base64(row.match_info).ok(),
    })
}

pub async fn db_get_game_states_for_match(
    _from: NodeIdentity,
    _arg: MatchRow2,
) -> anyhow::Result<Vec<GameStateRow2>> {
    let pool = get_pool().await?;

    let rows = sqlx::query!(
        r#"
SELECT game_type, user_id, start_time, game_seed, score, recv_time, data_version, last_action, state_data 
FROM game_states 
WHERE game_type = ? AND start_time = ? AND game_seed = ? 
ORDER BY user_id, recv_time
        "#,
        _arg.game_type,
        _arg.start_time,
        _arg.game_seed
    )
    .fetch_all(pool)
    .await?;

    let mut v = vec![];
    for row in rows {
        v.push(GameStateRow2 {
            game_type: row.game_type,
            user_id: row.user_id,
            start_time: row.start_time,
            game_seed: row.game_seed,
            score: row.score,
            recv_time: row.recv_time,
            data_version: row.data_version,
            last_action: row.last_action,
            state_data: deserialize_base64(row.state_data).ok(),
        });
    }
    Ok(v)
}

pub async fn get_last_game_states_for_match(
    _from: NodeIdentity,
    _match: GameMatch<NodeIdentity>,
) -> anyhow::Result<Vec<GameState>> {
    let mut v = vec![];

    for user in _match.users.iter() {
        v.push(
            get_last_game_state_for_match_and_user(_match.clone(), *user)
                .await?,
        );
    }
    Ok(v)
}

async fn get_last_game_state_for_match_and_user(
    _match: GameMatch<NodeIdentity>,
    user_id: NodeIdentity,
) -> anyhow::Result<GameState> {
    let game_type = format!("{:?}", _match.type_);
    let start_time = _match.time;
    let game_seed = serialize_base64(&_match.seed)?;
    let user_id_str = serialize_base64(&user_id.user_id().as_bytes())?;

    let pool = get_pool().await?;
    let row = sqlx::query!(
        r#"
SELECT state_data FROM game_states
WHERE game_type = ?
  AND start_time = ?
  AND game_seed = ?
  AND user_id = ?
ORDER BY recv_time DESC
LIMIT 1
        "#,
        game_type,
        start_time,
        game_seed,
        user_id_str
    )
    .fetch_optional(pool)
    .await?
    .context("no data found!")?;

    let state: GameState = deserialize_base64(row.state_data)?;
    Ok(state)
}

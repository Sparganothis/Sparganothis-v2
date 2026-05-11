use game::api::game_match::GameMatch;
use protocol::user_identity::NodeIdentity;
use tracing::info;
use crate::server::db2::get_pool;
use crate::server::db2::guest_login::serialize_base64;

pub async fn db_send_new_game(
    _from: NodeIdentity,
    _match: GameMatch<NodeIdentity>,
) -> anyhow::Result<()> {
    let user_id = serialize_base64(_from.user_id().as_bytes())?;
    let game_seed = serialize_base64(&_match.seed)?;
    let game_type = format!("{:?}", _match.type_);
    let match_info = serialize_base64(&_match)?;

    info!("INSERT NEW GAME!");
    let pool = get_pool().await?;

    sqlx::query!(
        r#"
INSERT IGNORE INTO games (game_type, user_id, start_time, game_seed, data_version, match_info)
VALUES (?, ?, ?, ?, ?, ?)
        "#,
        game_type,
        user_id,
        _match.time,
        game_seed,
        0i64,
        match_info
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn db_send_new_match(
    _from: NodeIdentity,
    (_match,): (GameMatch<NodeIdentity>,),
) -> anyhow::Result<()> {
    tracing::warn!("\n\n db_send_new_match !!!! \n\n !");

    db_send_new_game(_from.clone(), _match.clone()).await?;

    if _from != _match.users[0] {
        info!("Skipping db_send_match for non-first identity");
        return anyhow::Ok(());
    }

    let game_seed = serialize_base64(&_match.seed)?;
    let game_type = format!("{:?}", _match.type_);
    
    let user_ids: Vec<String> = _match.users.iter()
        .map(|u| serialize_base64(&u.user_id().as_bytes()))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let user_ids_serialized = serialize_base64(&user_ids)?;
    
    let match_info = serialize_base64(&_match)?;

    info!("INSERT NEW MATCH!");
    let pool = get_pool().await?;

    sqlx::query!(
        r#"
INSERT IGNORE INTO matches (game_type, start_time, user_ids, game_seed, match_id, data_version, match_info)
VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        game_type,
        _match.time,
        user_ids_serialized,
        game_seed,
        _match.match_id.to_string(),
        0i64,
        match_info
    )
    .execute(pool)
    .await?;

    Ok(())
}

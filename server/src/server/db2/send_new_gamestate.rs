use game::{
    api::game_match::GameMatch, tet::GameState, timestamp::get_timestamp_now_ms,
};
use protocol::user_identity::NodeIdentity;
use crate::server::db2::get_pool;
use crate::server::db2::guest_login::serialize_base64;

pub async fn db_send_new_gamestate(
    _from: NodeIdentity,
    (_match, game_state): (GameMatch<NodeIdentity>, GameState),
) -> anyhow::Result<()> {
    let user_id = serialize_base64(_from.user_id().as_bytes())?;
    let state_data = serialize_base64(&game_state)?;
    let game_seed = serialize_base64(&_match.seed)?;
    let game_type = format!("{:?}", _match.type_);
    let last_action = game_state.last_action.name();
    let is_finished = game_state.game_over_reason.is_some();
    let game_over_reason = format!("{:?}", game_state.game_over_reason);

    let pool = get_pool().await?;

    sqlx::query!(
        r#"
INSERT IGNORE INTO game_states (
    game_type, user_id, start_time, game_seed, recv_time, 
    score, data_version, last_action, state_data, is_finished, game_over_reason
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        game_type,
        user_id,
        _match.time,
        game_seed,
        get_timestamp_now_ms(),
        game_state.score as i64,
        0i64,
        last_action,
        state_data,
        is_finished,
        game_over_reason
    )
    .execute(pool)
    .await?;

    Ok(())
}

use anyhow::Context;
use game::api::game_match::GameMatchType;
use game::{
    api::game_match::GameMatch, tet::GameState, timestamp::get_timestamp_now_ms,
};
use protocol::user_identity::NodeIdentity;
use crate::server::db2::get_pool;
use crate::server::db2::guest_login::serialize_base64;
use crate::server::elo::compute_elo;

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

    let recv_time = get_timestamp_now_ms();
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
        recv_time,
        game_state.score as i64,
        0i64,
        last_action,
        state_data,
        is_finished,
        game_over_reason,
    )
    .execute(pool)
    .await?;


    if is_finished{
        let match_id = _match.match_id.to_string();
        let elo_score_percent = game_state.game_over_reason.context("bad item")?.elo_score_percent();

        let own_elo = get_elo(user_id.clone(), game_type.clone()).await;
        let opponent_elo = get_opponent_avg_elo(_from,
            game_type.clone(), _match.clone()).await;

        let opponent_score_percent = 100 - elo_score_percent;

        let (new_elo, outcome_txt) = compute_elo(elo_score_percent, own_elo, opponent_elo);


        sqlx::query!(
        r#"
INSERT IGNORE INTO match_outcomes (
    game_type,
     user_id, 
     start_time, 
     game_seed, 
     match_id, 
     recv_time, 
    score, 
    data_version, 
    last_action, 
    state_data, 
    is_finished, 
    game_over_reason, 
    elo_score_percent, 
            own_elo,
        opponent_score_percent,
        opponent_elo,
        new_elo
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,?,?,?)
        "#,
        game_type,
        user_id,
        _match.time,
        game_seed,
        match_id,
        recv_time,
        game_state.score as i64,
        0i64,
        last_action,
        state_data,
        is_finished,
        game_over_reason,
        elo_score_percent, 
        own_elo,
        opponent_score_percent,
        opponent_elo,
        new_elo

    )
    .execute(pool)
    .await?;

    
        let _ = set_elo(user_id, game_type, match_id, new_elo, recv_time, outcome_txt).await;

    }


    Ok(())
}



const DEFAULT_ELO: f64 = 1000.0;


async fn get_elo(user_id: String, game_type: String) -> f64  {
    _get_elo(user_id, game_type).await.unwrap_or(DEFAULT_ELO)
}

async fn _get_elo(user_id: String, game_type: String) -> anyhow::Result<f64>  {
    let pool = get_pool().await?;

    let record = sqlx::query!(
        r#"SELECT current_elo FROM user_elo_current WHERE game_type=? AND user_id=?"#,
       & game_type, &user_id)
        
        
    .fetch_one(pool)
        .await?;
   Ok(record.current_elo)

}

// async fn _get_elo(user_id: String, game_type: String) -> anyhow::Result<f64>  {

//     let pool = get_pool().await?;
//     let r = query!("SELECT elo FROM ")

// }


async fn get_opponent_avg_elo(our_node_id: NodeIdentity, game_type: String, match_info: GameMatch<NodeIdentity>) -> f64 {
    
    let opponents = match_info.users.into_iter()
    .filter(|x| *x != our_node_id)
    .map(
        |x| serialize_base64(x.user_id().as_bytes()).unwrap_or("".to_string())
    ).collect::<Vec<_>>();

    let mut opponents_elos = vec![];

    for o in opponents {
        if let Ok(elo) = _get_elo(o, game_type.clone()).await {
            opponents_elos.push(elo);
        }
    }
    if opponents_elos.is_empty() {
        opponents_elos.push(DEFAULT_ELO);
    }

    let n = opponents_elos.len();

    let s: f64 = opponents_elos.into_iter().sum();

    s  / n as f64
}



#[tracing::instrument()]
async fn set_elo(user_id: String, game_type: String, match_id: String, new_elo: f64, recv_time: i64, outcome_txt: String) -> anyhow::Result<()> {
let pool = get_pool().await?;
        sqlx::query!(
        r#"
REPLACE  INTO user_elo_current (

    game_type,
     user_id, 
     match_id,

     current_elo,
     recv_time, 
    data_version
   
)
VALUES (?, ?, ?,  ?, ?, ?)

        "#,
        game_type,
        user_id,
        match_id,
        new_elo,
        recv_time,
        0i64,

    )
    .execute(pool)
    .await?;

            sqlx::query!(
        r#"
INSERT IGNORE INTO user_elo_history (

    game_type,
     user_id, 
     match_id,

     recv_time, 
     new_elo,
    data_version,
    elo_outcome
   
)
VALUES (?, ?, ?, ?, ?, ?, ?)

        "#,
        game_type,
        user_id,
        match_id,
        recv_time,
        new_elo,
        0i64,
        outcome_txt,
    )
    .execute(pool)
    .await?;

    Ok(())

}
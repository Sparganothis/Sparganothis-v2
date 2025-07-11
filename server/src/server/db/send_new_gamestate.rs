use clickhouse::Row;
use game::{api::game_match::GameMatch, tet::GameState};
use protocol::user_identity::NodeIdentity;
use serde::{Deserialize, Serialize};
use crate::server::db::{
    clickhouse_client::get_clickhouse_client, guest_login::serialize_base64,
};

#[derive(Row, Serialize, Deserialize)]
pub struct GameStateRow {
    pub game_type: String,
    pub user_id: String,
    pub start_time: i64,
    pub game_seed: String,
    pub state_idx: i64,

    pub data_version: i64,
    pub last_action: String,
    pub state_data: String,
}

pub async fn db_send_new_gamestate(
    _from: NodeIdentity,
    (_match, game_state): (GameMatch<NodeIdentity>, GameState),
) -> anyhow::Result<()> {
    let user_id = _from.user_id().as_bytes();
    let user_id = serialize_base64(user_id)?;
    let state_data = serialize_base64(&game_state)?;

    let game_seed = _match.seed;
    let game_seed = serialize_base64(&game_seed)?;
    let new_row = GameStateRow {
        game_type: format!("{:?}", _match.type_),
        start_time: _match.time,
        user_id,
        game_seed,
        data_version: 0,
        state_idx: game_state.score as i64,
        last_action: game_state.last_action.name(),
        state_data,
    };

    // info!("INSERT NEW GAMESTTEA!");
    let client = get_clickhouse_client();
    let mut insert = client.insert("game_states")?;
    insert.write(&new_row).await?;
    insert.end().await?;

    // info!("INSRT GAMESTTEA OK!");

    Ok(())
}

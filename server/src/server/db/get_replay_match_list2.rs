use anyhow::Context;
use base64::Engine;
use game::{api::game_match::GameMatch, timestamp::get_timestamp_now_ms};
use protocol::server_chat_api::api_declarations::GameStateRow2;
use protocol::{
    server_chat_api::api_declarations::MatchRow2, user_identity::NodeIdentity,
};

use crate::server::db::send_new_gamestate::GameStateRow;
use crate::server::db::{
    clickhouse_client::get_clickhouse_client, guest_login::deserialize_base64,
    send_new_match::MatchRow,
};

use clickhouse::{sql::Identifier, Row};
use serde::{Deserialize, Serialize};

pub async fn db_get_list_matches(
    from: NodeIdentity,
    _arg: (),
) -> anyhow::Result<Vec<MatchRow2>> {
    tracing::info!("DB ADD GUEST LOGIN for user = {:?}", from);

    let client = get_clickhouse_client();
    let all_matches = {
        let cursor = client
            .query("SELECT ?fields FROM ?")
            .bind(Identifier("matches"))
            .fetch_all::<MatchRow>()
            .await?;
        let count = cursor;
        count
    };

    let all2 = all_matches
        .into_iter()
        .map(|i| MatchRow2 {
            game_type: i.game_type,
            start_time: i.start_time,
            user_ids: i.user_ids,
            game_seed: i.game_seed,
            match_id: i.match_id,
            data_version: i.data_version,
            match_info: deserialize_base64(i.match_info).ok(),
        })
        .collect();

    Ok(all2)
}

pub async fn db_get_detail_match(
    from: NodeIdentity,
    _arg: String,
) -> anyhow::Result<MatchRow2> {
    tracing::info!("DB ADD GUEST LOGIN for user = {:?}", from);

    let client = get_clickhouse_client();
    let all_matches = {
        let cursor = client
            .query("SELECT ?fields FROM ? WHERE match_id = ?")
            .bind(Identifier("matches"))
            .bind(_arg)
            .fetch_all::<MatchRow>()
            .await?;
        let count = cursor;
        count
    };

    let all2 = all_matches
        .into_iter()
        .map(|i| MatchRow2 {
            game_type: i.game_type,
            start_time: i.start_time,
            user_ids: i.user_ids,
            game_seed: i.game_seed,
            match_id: i.match_id,
            data_version: i.data_version,
            match_info: deserialize_base64(i.match_info).ok(),
        })
        .collect::<Vec<_>>();
    let all2 = all2.get(0).context("no result!")?;
    Ok(all2.clone())
}

pub async fn db_get_game_states_for_match(
    _from: NodeIdentity,
    _arg: MatchRow2,
) -> anyhow::Result<Vec<GameStateRow2>> {
    let client = get_clickhouse_client();

    let all_matches = {
        tracing::warn!(
            " \n\n
        SELECT * FROM game_states WHERE game_type = '{}'
         AND start_time = '{}'
          AND game_seed = '{}'
           ORDER BY user_id, state_idx
        \n\n",
            _arg.game_type,
            _arg.start_time,
            _arg.game_seed
        );

        let cursor = client
            .query("SELECT ?fields FROM ? WHERE game_type = ? AND start_time = ? AND game_seed = ? ORDER BY user_id, state_idx")
            .bind(Identifier("game_states"))
            .bind(_arg.game_type)
            .bind(_arg.start_time)
            .bind(_arg.game_seed)
            .fetch_all::<GameStateRow>()
            .await?;
        let count = cursor;
        count
    };

    let all2 = all_matches
        .into_iter()
        .map(|i| GameStateRow2 {
            // game_type: i.game_type,
            // start_time: i.start_time,
            // user_ids: i.user_ids,
            // game_seed: i.game_seed,
            // match_id: i.match_id,
            // data_version: i.data_version,
            // match_info: deserialize_base64(i.match_info).ok(),
            game_type: i.game_type,
            user_id: i.user_id,
            start_time: i.start_time,
            game_seed: i.game_seed,
            state_idx: i.state_idx,

            data_version: i.data_version,
            last_action: i.last_action,
            state_data: deserialize_base64(i.state_data).ok(),
        })
        .collect::<Vec<_>>();
    Ok(all2)
}

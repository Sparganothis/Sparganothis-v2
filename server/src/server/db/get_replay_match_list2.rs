use anyhow::Context;
use game::api::game_match::GameMatch;
use game::tet::GameState;
use protocol::server_chat_api::api_declarations::GameStateRow2;
use protocol::{
    server_chat_api::api_declarations::MatchRow2, user_identity::NodeIdentity,
};

use crate::server::db::guest_login::serialize_base64;
use crate::server::db::send_new_gamestate::GameStateRow;
use crate::server::db::{
    clickhouse_client::get_clickhouse_client, guest_login::deserialize_base64,
    send_new_match::MatchRow,
};

use clickhouse::sql::Identifier;

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
           ORDER BY user_id, recv_time
        \n\n",
            _arg.game_type,
            _arg.start_time,
            _arg.game_seed
        );

        let cursor = client
            .query("SELECT ?fields FROM ? WHERE game_type = ? AND start_time = ? AND game_seed = ? ORDER BY user_id, recv_time")
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
            score: i.score,
            recv_time: i.recv_time,

            data_version: i.data_version,
            last_action: i.last_action,
            state_data: deserialize_base64(i.state_data).ok(),
        })
        .collect::<Vec<_>>();
    Ok(all2)
}


pub async fn get_last_game_states_for_match(_from: NodeIdentity, _match: GameMatch<NodeIdentity>) -> anyhow::Result<Vec<GameState>> {
    let mut v = vec![];

    for user in _match.users.iter() {
        v.push(get_last_game_state_for_match_and_user(_match.clone(), user.clone()).await?);
    }
    Ok(v)
}

async fn get_last_game_state_for_match_and_user( _match: GameMatch<NodeIdentity>, user_id: NodeIdentity) -> anyhow::Result<GameState> {
    let game_type = format!("{:?}", _match.type_);
    let start_time = _match.time;
    let game_seed = _match.seed;
    let game_seed = serialize_base64(&game_seed)?;
    let user_id = serialize_base64(&user_id.user_id().as_bytes())?;

    let sql = r#"
        SELECT state_data FROM game_states
        WHERE game_type = ?
        AND start_time = ?
        AND game_seed = ?
        AND user_id = ?
        AND recv_time = (
            SELECT max(recv_time) as recv_time
            FROM game_states       
            WHERE game_type = ?
            AND start_time = ?
            AND game_seed = ?
            AND user_id = ?
        )
    "#;

    let client = get_clickhouse_client();
    let data = client.query(sql)
    .bind(&game_type).bind(start_time).bind(&game_seed).bind(&user_id)
    .bind(&game_type).bind(start_time).bind(&game_seed).bind(&user_id)
    .fetch_all::<String>().await?;


    let x = data.into_iter().map(|s| {
        deserialize_base64::<GameState>(s)
    }).collect::<Result<Vec<_>,_>>()?;

    if x.is_empty() {
        anyhow::bail!("no data found!");
    }

    Ok(x.last().unwrap().clone())
}
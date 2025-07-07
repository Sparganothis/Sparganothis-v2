use anyhow::Context;
use base64::Engine;
use game::{api::game_match::GameMatch, timestamp::get_timestamp_now_ms};
use protocol::{impl_api_method, postcard, server_chat_api::api_declarations::MatchRow2, user_identity::NodeIdentity};

use crate::server::db::{clickhouse_client::get_clickhouse_client, guest_login::deserialize_base64, send_new_match::MatchRow};

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

    let all2 = all_matches.into_iter().map(|i| MatchRow2 {
        game_type: i.game_type,
        start_time: i.start_time,
        user_ids: i.user_ids,
        game_seed: i.game_seed,
        match_id: i.match_id,
        data_version: i.data_version,
        match_info: deserialize_base64(i.match_info).ok(),
    }).collect();

    Ok(all2)
}

use protocol::server_chat_api::api_declarations::GetReplayMatchList;
impl_api_method!(GetReplayMatchList, db_get_list_matches);

use anyhow::Context;
use game::{
    api::game_match::{GameMatch, GameMatchType},
    tet::get_random_seed,
    timestamp::get_timestamp_now_ms,
};
use protocol::user_identity::NodeIdentity;

use crate::server::{
    db::guest_login::{deserialize_base64, serialize_base64},
    multiplayer::matchmaker::matchmaker_basic::{
        get_lock_values, increment_redis_counter, set_lock,
    },
};

use super::matchmaker_basic::run_basic_multiplayer_matchmaker;

pub async fn run_multiplayer_matchmaker_1(
    _from: NodeIdentity,
    arg: GameMatchType,
) -> anyhow::Result<Vec<NodeIdentity>> {
    /* RATE LIMIT 1 matchmaking / 5s / player */
    let userid_str = _from.user_id().as_bytes().clone();
    let userid_str = serialize_base64(&userid_str)?;
    let userid_str = format!("_userid_ratelimit_{userid_str}");
    let _l = set_lock(&userid_str, &get_timestamp_now_ms().to_string(), 5000).await.context("limited by too many matchmaking requests for your user id (pls wait 5s)")?;

    let identity_str = serialize_base64(&_from)?;
    let game_type_string = format!("{arg:?}");

    let n = GameMatchType::get_match_num_players(&Some(arg.clone()));
    let mut _basic =
        run_basic_multiplayer_matchmaker(identity_str, &game_type_string, n)
            .await?;
    _basic.sort();
    let match_identities = _basic
        .iter()
        .map(|x| deserialize_base64(x.to_string()))
        .collect::<anyhow::Result<Vec<NodeIdentity>>>()?;
    return Ok(match_identities);
}

pub async fn run_multiplayer_matchmaker_2(
    _from: NodeIdentity,
    (arg, match_identities): (GameMatchType, Vec<NodeIdentity>),
) -> anyhow::Result<GameMatch<NodeIdentity>> {
    let combo_key = match_identities
        .iter()
        .map(|x| serialize_base64(x))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let combo_key = combo_key.join("_");
    let n = GameMatchType::get_match_num_players(&Some(arg.clone()));

    let proposed_match = GameMatch {
        type_: arg.clone(),
        match_id: uuid::Uuid::new_v4(),
        seed: get_random_seed(),
        time: get_timestamp_now_ms(),
        users: match_identities.clone(),
        title: format!("{arg:?} - {match_identities:?}"),
    };
    let proposed_match = serialize_base64(&proposed_match)?;

    let counter_key = format!("match_counter_{combo_key}");
    let info_key = format!("match_info_{combo_key}");
    let _z = set_lock(&info_key, &proposed_match, 3000).await;
    let counter = increment_redis_counter(&counter_key, 400).await?;
    if counter != n as i32 {
        anyhow::bail!("final match counter is not = n");
    }
    let val = get_lock_values(vec![info_key]).await?;
    let val = val.get(0).context("no match in lockfile!")?.to_string();
    let val: GameMatch<NodeIdentity> = deserialize_base64(val)?;
    tracing::debug!(
        "Matchmaking for {} finished - {}",
        _from.nickname(),
        val.match_id
    );
    Ok(val)
}

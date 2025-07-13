use clickhouse::Row;
use iroh::PublicKey;
use protocol::{
    server_chat_api::api_declarations::UserProfileListItem,
    user_identity::{NodeIdentity, UserIdentity},
};
use serde::Deserialize;

use crate::server::db::{
    clickhouse_client::get_clickhouse_client, guest_login::{deserialize_base64, serialize_base64},
};

const SELECT_TOP_PLAYERS_BY_GAMES: &'static str = r#"

SELECT users.user_id as user_id, users.nickname, users.first_login, events.last_login, game_count.game_count
FROM sparganothis.guest_users users
INNER JOIN (SELECT user_id, max(last_login) as last_login FROM sparganothis.guest_user_login_events GROUP BY user_id) events
ON users.user_id = events.user_id
LEFT OUTER   JOIN (SELECT user_id, count() as game_count FROM sparganothis.games GROUP BY user_id) game_count
ON users.user_id = game_count.user_id

ORDER BY game_count desc
LIMIT 100

"#;

#[derive(Row, Deserialize, Debug)]
#[allow(dead_code)]
struct TopPlayersRow {
    user_id: String,
    nickname: String,
    first_login: i64,
    last_login: i64,
    game_count: u64,
}

pub async fn db_get_users_with_top_game_counts(
    _from: NodeIdentity,
    _arg: (),
) -> anyhow::Result<Vec<UserProfileListItem>> {
    let client = get_clickhouse_client();
    let users = client
        .query(SELECT_TOP_PLAYERS_BY_GAMES)
        .fetch_all::<TopPlayersRow>()
        .await?;
    let mut v = vec![];
    for u in users {
        let b: [u8; 32] = deserialize_base64(u.user_id.clone())?;
        let b = PublicKey::from_bytes(&b)?;
        let b = UserIdentity::from_userid(b);

        v.push(UserProfileListItem {
            user: b,
            nickname: b.nickname(),
            first_login: u.first_login,
            last_login: u.last_login,
            game_count: u.game_count,
        })
    }

    Ok(v)
}


const SELECT_SINGLE_PLAYER: &'static str = r#"

SELECT users.user_id as user_id, users.nickname, users.first_login, events.last_login, game_count.game_count
FROM sparganothis.guest_users users
INNER JOIN (SELECT user_id, max(last_login) as last_login FROM sparganothis.guest_user_login_events GROUP BY user_id) events
ON users.user_id = events.user_id
LEFT OUTER   JOIN (SELECT user_id, count() as game_count FROM sparganothis.games GROUP BY user_id) game_count
ON users.user_id = game_count.user_id

WHERE users.user_id = ?
LIMIT 100

"#;

pub async fn db_get_user_profile(
    _from: NodeIdentity,
    _arg: UserIdentity,
) -> anyhow::Result<UserProfileListItem>
{
    let client = get_clickhouse_client();
    let userid_str = _arg.user_id().as_bytes();
    let userid_str = serialize_base64(userid_str)?;
    let users = client
        .query(SELECT_SINGLE_PLAYER).bind(userid_str)
        .fetch_all::<TopPlayersRow>()
        .await?;
    for u in users {
        let b: [u8; 32] = deserialize_base64(u.user_id.clone())?;
        let b = PublicKey::from_bytes(&b)?;
        let b = UserIdentity::from_userid(b);

        return Ok(UserProfileListItem {
            user: b,
            nickname: b.nickname(),
            first_login: u.first_login,
            last_login: u.last_login,
            game_count: u.game_count,
        });
    }
    anyhow::bail!("user not found!")
}
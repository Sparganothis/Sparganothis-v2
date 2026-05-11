use crate::server::db2::get_pool;
use crate::server::db2::guest_login::{deserialize_base64, serialize_base64};
use anyhow::Context;
use iroh::PublicKey;
use protocol::{
    api::api_declarations::UserProfileListItem,
    user_identity::{NodeIdentity, UserIdentity},
};

pub async fn db_get_users_with_top_game_counts(
    _from: NodeIdentity,
    _arg: (),
) -> anyhow::Result<Vec<UserProfileListItem>> {
    let pool = get_pool().await?;

    // MariaDB query adapted from ClickHouse.
    // Both guest_users.user_id and games.user_id are now VARCHAR (base64 strings).
    let rows = sqlx::query!(
        r#"
SELECT 
    u.user_id as user_id, 
    u.nickname, 
    u.first_login, 
    e.last_login, 
    IFNULL(gc.game_count, 0) as game_count
FROM guest_users u
INNER JOIN (
    SELECT user_id, MAX(last_login) as last_login 
    FROM guest_user_login_events 
    GROUP BY user_id
) e ON u.user_id = e.user_id
LEFT OUTER JOIN (
    SELECT user_id, COUNT(*) as game_count 
    FROM games 
    GROUP BY user_id
) gc ON u.user_id = gc.user_id
ORDER BY game_count DESC
LIMIT 100
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut v = vec![];
    for row in rows {
        let b: [u8; 32] = deserialize_base64(row.user_id)?;
        let pubkey = PublicKey::from_bytes(&b)?;
        let identity = UserIdentity::from_userid(pubkey);
        let last_login = row.last_login.unwrap_or(0);

        v.push(UserProfileListItem {
            user: identity,
            nickname: row.nickname,
            first_login: row.first_login,
            last_login,
            game_count: row.game_count as u64,
        })
    }

    Ok(v)
}

pub async fn db_get_user_profile(
    _from: NodeIdentity,
    _arg: UserIdentity,
) -> anyhow::Result<UserProfileListItem> {
    let pool = get_pool().await?;
    let user_id_str = serialize_base64(&_arg.user_id().as_bytes())?;

    let row = sqlx::query!(
        r#"
SELECT 
    u.user_id as user_id, 
    u.nickname, 
    u.first_login, 
    e.last_login, 
    IFNULL(gc.game_count, 0) as game_count
FROM guest_users u
INNER JOIN (
    SELECT user_id, MAX(last_login) as last_login 
    FROM guest_user_login_events 
    GROUP BY user_id
) e ON u.user_id = e.user_id
LEFT OUTER JOIN (
    SELECT user_id, COUNT(*) as game_count 
    FROM games 
    GROUP BY user_id
) gc ON u.user_id = gc.user_id
WHERE u.user_id = ?
LIMIT 1
        "#,
        user_id_str
    )
    .fetch_optional(pool)
    .await?
    .context("user not found!")?;

    let b: [u8; 32] = deserialize_base64(row.user_id)?;
    let pubkey = PublicKey::from_bytes(&b)?;
    let identity = UserIdentity::from_userid(pubkey);
        let last_login = row.last_login.unwrap_or(0);

    Ok(UserProfileListItem {
        user: identity,
        nickname: row.nickname,
        first_login: row.first_login,
        last_login,
        game_count: row.game_count as u64,
    })
}

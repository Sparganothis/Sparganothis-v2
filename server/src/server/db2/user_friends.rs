use game::timestamp::get_timestamp_now_ms;
use iroh::PublicKey;
use protocol::{
    api::api_declarations::FriendInfo,
    user_identity::{NodeIdentity, UserIdentity},
};
use tracing::info;
use crate::server::db2::get_pool;
use crate::server::db2::guest_login::{serialize_base64, deserialize_base64};

pub async fn user_add_friend(
    _from: NodeIdentity,
    _arg: UserIdentity,
) -> anyhow::Result<()> {
    let user_id = serialize_base64(_from.user_id().as_bytes())?;
    let friend_id = serialize_base64(_arg.user_id().as_bytes())?;
    let now = get_timestamp_now_ms();

    info!("INSERT NEW FRIEND!");
    let pool = get_pool().await?;

    sqlx::query!(
        r#"
INSERT IGNORE INTO user_friends (user_id, friend_id, added_on, data_version)
VALUES (?, ?, ?, ?)
        "#,
        user_id,
        friend_id,
        now,
        0i64
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn user_delete_friend(
    _from: NodeIdentity,
    _arg: UserIdentity,
) -> anyhow::Result<()> {
    let user_id = serialize_base64(_from.user_id().as_bytes())?;
    let friend_id = serialize_base64(_arg.user_id().as_bytes())?;

    let pool = get_pool().await?;
    sqlx::query!(
        r#"
DELETE FROM user_friends 
WHERE user_id = ? AND friend_id = ?
        "#,
        user_id,
        friend_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn _friend_exists(user_id: &str, friend_id: &str) -> anyhow::Result<bool> {
    let pool = get_pool().await?;
    let row = sqlx::query!(
        r#"
SELECT count(*) as count FROM user_friends 
WHERE user_id = ? AND friend_id = ?
        "#,
        user_id,
        friend_id
    )
    .fetch_one(pool)
    .await?;
    
    Ok(row.count > 0)
}

pub async fn user_list_friends(
    _from: NodeIdentity,
    _arg: (),
) -> anyhow::Result<Vec<FriendInfo>> {
    let user_id = serialize_base64(_from.user_id().as_bytes())?;
    let pool = get_pool().await?;

    let rows = sqlx::query!(
        r#"
SELECT friend_id, added_on FROM user_friends 
WHERE user_id = ?
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let mut v = vec![];
    for row in rows {
        let b: [u8; 32] = deserialize_base64(row.friend_id)?;
        let pubkey = PublicKey::from_bytes(&b)?;
        let identity = UserIdentity::from_userid(pubkey);

        v.push(FriendInfo {
            friend_id: identity,
            added_on: row.added_on,
        });
    }

    Ok(v)
}

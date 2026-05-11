use anyhow::Context;
use base64::Engine;
use game::timestamp::get_timestamp_now_ms;
use protocol::{postcard, user_identity::NodeIdentity};
use serde::{Deserialize, Serialize};
use crate::server::db2::get_pool;

pub fn serialize_base64<T: Serialize>(t: &T) -> anyhow::Result<String> {
    let vec = postcard::to_stdvec(t)?;
    let v = base64::prelude::BASE64_URL_SAFE.encode(vec);
    Ok(v)
}

pub fn deserialize_base64<T: for<'a> Deserialize<'a>>(
    base64: String,
) -> anyhow::Result<T> {
    let vec = base64::prelude::BASE64_URL_SAFE.decode(base64)?;
    let obj = postcard::from_bytes(&vec)?;
    Ok(obj)
}

pub async fn db_add_guest_login(
    from: NodeIdentity,
    _arg: (),
) -> anyhow::Result<()> {
    tracing::info!("DB ADD GUEST LOGIN for user = {:?}", from);

    let pool = get_pool().await?;

    let user_id = serialize_base64(from.user_id().as_bytes())?;
    let nickname = from.nickname();
    let now = get_timestamp_now_ms();

    // Insert or ignore into guest_users
    sqlx::query!(
        r#"
INSERT IGNORE INTO guest_users (user_id, nickname, first_login, data_version)
VALUES (?, ?, ?, ?)
        "#,
        user_id,
        nickname,
        now,
        0i64
    )
    .execute(pool)
    .await?;

    // Insert into guest_user_login_events
    sqlx::query!(
        r#"
INSERT IGNORE INTO guest_user_login_events (user_id, last_login)
VALUES (?, ?)
        "#,
        user_id,
        now
    )
    .execute(pool)
    .await?;

    Ok(())
}

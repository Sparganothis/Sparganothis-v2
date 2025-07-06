use anyhow::Context;
use base64::Engine;
use game::timestamp::get_timestamp_now_ms;
use protocol::{impl_api_method, user_identity::NodeIdentity};

use crate::server::db::clickhouse_client::get_clickhouse_client;

    use serde::Serialize;
use clickhouse::{sql::Identifier, Row};

#[derive(Row, Serialize)]
struct GuestUser {
    user_id: String,
    nickname: String,
    first_login: i64,
    data_version: i64,
}


#[derive(Row, Serialize)]
struct GuestUserLoginEvent {
    user_id: String,
    last_login: i64,
}


pub async fn db_add_guest_login(from: NodeIdentity, _arg: ()) -> anyhow::Result<()> {
    tracing::info!("DB ADD GUEST LOGIN for user = {:?}", from);

    let client = get_clickhouse_client();

    let user_id = *from.user_id().as_bytes();
    let user_id = base64::prelude::BASE64_URL_SAFE.encode(user_id);



    let new_guest_user = GuestUser {
        user_id: user_id.clone(),
        nickname: from.nickname(),
        first_login: get_timestamp_now_ms(),
        data_version: 0,
    };
    let guest_login_event = GuestUserLoginEvent {
        user_id: user_id.clone(),
        last_login: get_timestamp_now_ms(),
    };

    // select
    let user_count = {
        let  cursor = client.query( "SELECT count() FROM ? WHERE user_id = ?",        ).bind(Identifier("guest_users"))
        .bind(user_id.clone()).fetch_all::<u64>().await?;
        let count = *cursor.get(0).context("no count row??")?;
        count
    };


    if user_count == 0 {
        let mut insert = client.insert("guest_users")?;
        insert.write(&new_guest_user).await?;
        insert.end().await?;
    }

    let mut insert = client.insert("guest_user_login_events ")?;
    insert.write(&guest_login_event).await?;
    insert.end().await?;


    Ok(())
}

use protocol::server_chat_api::api_methods::LoginApiMethod;
impl_api_method!(LoginApiMethod, db_add_guest_login);

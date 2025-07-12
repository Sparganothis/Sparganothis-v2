use clickhouse::{sql::Identifier, Row};
use game::timestamp::get_timestamp_now_ms;
use iroh::PublicKey;
use protocol::{server_chat_api::api_declarations::FriendInfo, user_identity::{NodeIdentity, UserIdentity}};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::server::db::{clickhouse_client::get_clickhouse_client, guest_login::{deserialize_base64, serialize_base64}};

pub async fn user_add_friend(_from: NodeIdentity, _arg: UserIdentity) -> anyhow::Result<()> {
    let userid_str = _from.user_id().as_bytes().clone();
    let userid_str = serialize_base64(&userid_str)?;

    let arg_userid = _arg.user_id().as_bytes().clone();
    let arg_userid =serialize_base64(&arg_userid)?;

    let friend_exists = friend_exists(&userid_str, &arg_userid).await?;
    if friend_exists {
        return Ok(());
    }

    info!("INSERT NEW FRIEND!");
    let new_friend = UserFriendRow {
        user_id: userid_str.clone(),
        friend_id: arg_userid.clone(),
        added_on: get_timestamp_now_ms(),
        data_version: 0,
    };
    let client = get_clickhouse_client();
    let mut insert = client.insert("user_friends")?;
    insert.write(&new_friend).await?;
    insert.end().await?;

    Ok(())
}
pub async fn user_delete_friend(_from: NodeIdentity, _arg: UserIdentity) -> anyhow::Result<()> {
    let userid_str = _from.user_id().as_bytes().clone();
    let userid_str = serialize_base64(&userid_str)?;

    let arg_userid = _arg.user_id().as_bytes().clone();
    let arg_userid =serialize_base64(&arg_userid)?;

    let friend_exists = friend_exists(&userid_str, &arg_userid).await?;
    if !friend_exists {
        return Ok(());
    }

    let client = get_clickhouse_client();
    let q = client.query("ALTER TABLE ? DELETE WHERE user_id = ? AND friend_id = ?")
    .bind(Identifier("user_friends")).bind(userid_str).bind(arg_userid);
    q.execute().await?;

    Ok(())
}

async fn friend_exists(user_id: &str, friend_id: &str) -> anyhow::Result<bool> {
    let client = get_clickhouse_client();
    let user_count = {
        client
            .query("SELECT count() FROM ? WHERE user_id = ? AND friend_id = ?")
            .bind(Identifier("user_friends"))
            .bind(user_id)
            .bind(friend_id)
            .fetch_all::<u64>()
            .await?[0]
    };
    Ok(user_count == 1)
}
pub async fn user_list_friends(_from: NodeIdentity, _arg: ()) -> anyhow::Result<Vec<FriendInfo>> {
    let userid_str = _from.user_id().as_bytes().clone();
    let userid_str = serialize_base64(&userid_str)?;
    
    let client = get_clickhouse_client();    
    let all = client
            .query("SELECT ?fields FROM ? WHERE user_id = ?")
            .bind(Identifier("user_friends"))
            .bind(userid_str)
            .fetch_all::<UserFriendRow>()
            .await?;

    let mut v = vec![];
    for x in all {
        let b: [u8; 32] = deserialize_base64(x.friend_id.clone())?;
        let b = PublicKey::from_bytes(&b)?;
        let b = UserIdentity::from_userid(b);

        v.push(FriendInfo {
            friend_id: b,
            added_on: x.added_on,
        });
    }

    Ok(v)
}

#[derive(Row, Serialize, Deserialize)]
pub struct UserFriendRow {
    pub user_id: String,
    pub friend_id: String,
    pub added_on: i64,
    pub data_version: i64,
}
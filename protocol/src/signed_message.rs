use std::sync::Arc;

use crate::{
    datetime_now,
    user_identity::{NodeIdentity, UserIdentitySecrets},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use iroh::{PublicKey, SecretKey};
use iroh_base::Signature;
use serde::{Deserialize, Serialize};

pub trait AcceptableType:
    serde::Serialize
    + for<'a> serde::Deserialize<'a>
    + Clone
    + std::fmt::Debug
    + PartialEq
    + Send
    + Sync
    + 'static
{
}
impl<T> AcceptableType for T where
    T: serde::Serialize
        + for<'a> serde::Deserialize<'a>
        + Clone
        + std::fmt::Debug
        + PartialEq
        + Send
        + Sync
        + 'static
{
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    node_pubkey: PublicKey,
    user_pubkey: PublicKey,
    data: Vec<u8>,
    node_signature: Signature,
    user_signature: Signature,
}

impl SignedMessage {
    pub fn verify_and_decode<T: AcceptableType>(
        bytes: &[u8],
    ) -> Result<WireMessage<T>> {
        let signed_message: Self = postcard::from_bytes(bytes)?;
        let message: WireMessage<T> =
            postcard::from_bytes(&signed_message.data)?;
        // let signed_message: Self = bincode::deserialize(bytes)?;
        // let message: WireMessage<T> =
        // bincode::deserialize(&signed_message.data)?;

        if message.from.user_id() != &signed_message.user_pubkey {
            return Err(anyhow::anyhow!("user id mismatch"));
        }
        if message.from.node_id() != &signed_message.node_pubkey {
            return Err(anyhow::anyhow!("node id mismatch"));
        }

        signed_message
            .node_pubkey
            .verify(&signed_message.data, &signed_message.node_signature)?;
        signed_message
            .user_pubkey
            .verify(&signed_message.data, &signed_message.user_signature)?;

        Ok(message)
    }
}

#[derive(Debug, Clone)]
pub struct MessageSigner {
    pub(crate) node_secret_key: Arc<SecretKey>,
    pub(crate) user_secrets: Arc<UserIdentitySecrets>,
    pub(crate) node_identity: Arc<NodeIdentity>,
}

impl MessageSigner {
    pub fn sign_and_encode<T: AcceptableType>(
        &self,
        message: T,
    ) -> Result<(Vec<u8>, WireMessage<T>)> {
        let timestamp = datetime_now();
        let from: &NodeIdentity = &self.node_identity;
        let from = from.clone();
        let wire_message = WireMessage {
            _timestamp: timestamp,
            message: message.clone(),
            from,
            _message_id: uuid::Uuid::new_v4(),
        };
        let data = postcard::to_stdvec(&wire_message)?;
        // info!("WireMessage size: {:?}", data.len());
        // let compressed = deflate::deflate_bytes_conf(&data, deflate::Compression::Best);
        // info!("Compressed WireMessage size: {:?}", compressed.len());
        // let data = bincode::serialize(&wire_message)?;
        let node_signature = self.node_secret_key.sign(&data);
        let user_signature = self.user_secrets.secret_key().sign(&data);
        let signed_message = SignedMessage {
            node_pubkey: self.node_identity.node_id().clone(),
            user_pubkey: self.node_identity.user_id().clone(),
            data,
            node_signature,
            user_signature,
        };
        let encoded = postcard::to_stdvec(&signed_message)?;
        // info!("SignedMessage size: {:?}", encoded.len());
        // let compressed = deflate::deflate_bytes_conf(&encoded, deflate::Compression::Best);
        // info!("Compressed SignedMessage size: {:?}", compressed.len());
        // let encoded = bincode::serialize(&signed_message)?;
        Ok((encoded, wire_message))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WireMessage<T> {
    pub _timestamp: DateTime<Utc>,
    pub _message_id: uuid::Uuid,
    pub from: NodeIdentity,
    pub message: T,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ReceivedMessage<T: IChatRoomType> {
    pub _sender_timestamp: DateTime<Utc>,
    pub _received_timestamp: DateTime<Utc>,
    pub _message_id: uuid::Uuid,
    pub from: NodeIdentity,
    pub message: T::M,
}

pub trait IChatRoomType: AcceptableType {
    type M: AcceptableType;
    type P: AcceptableType;
    fn default_presence() -> Self::P;
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ChatMessage<T: IChatRoomType> {
    Presence { presence: T::P },
    Message { text: T::M },
}

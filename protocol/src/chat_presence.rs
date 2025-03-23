use iroh::NodeId;
use n0_future::time::Instant;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{Notify, RwLock};

use crate::{
    _const::{PRESENCE_EXPIRATION, PRESENCE_IDLE}, chat::ChatMessageType, user_identity::NodeIdentity
};

#[derive(Clone, Debug)]
pub struct ChatPresence<T: ChatMessageType> {
    presence: Arc<RwLock<ChatPresenceData<T>>>,
    notify: Arc<Notify>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum PresenceFlag {
    ACTIVE,
    IDLE,
    EXPIRED,
}

impl PresenceFlag {
    pub fn from_instant(instant: Instant) -> Self {
        let duration = instant.elapsed();
        if duration < PRESENCE_IDLE {
            Self::ACTIVE
        } else if duration < PRESENCE_EXPIRATION - PRESENCE_IDLE {
            Self::IDLE
        } else {
            Self::EXPIRED
        }
    }
}
pub type PresenceList<T> = Vec<(PresenceFlag, Instant, NodeIdentity, <T as ChatMessageType>::P)>;

impl<T: ChatMessageType> ChatPresence<T> {
    pub fn new() -> Self {
        Self {
            presence: Arc::new(RwLock::new(ChatPresenceData::default())),
            notify: Arc::new(Notify::new()),
        }
    }
    pub fn notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.notify.notified()
    }
    pub async fn add_presence(&self, identity: &NodeIdentity, payload: &T::P) {
        let identity = identity.clone();
        let now = Instant::now();
        let mut w = self.presence.write().await;
        let old_value = w.clone();
        w.map.insert(identity.node_id().clone(), (now, identity, payload.clone()));
        w.map.retain(|_, (last_seen, _, _)| {
            now.duration_since(*last_seen) < PRESENCE_EXPIRATION
        });
        let new_value = w.clone();
        if old_value != new_value {
            self.notify.notify_waiters();
        }
    }

    pub async fn get_presence_list(&self) -> PresenceList<T> {
        let p = self.presence.read().await.map.clone();
        let mut p = p.into_iter().collect::<Vec<_>>();
        p.sort_by_key(|(_, (_k, _userid, _payload))| {
            (
                _userid.user_id().to_string(),
                _userid.nickname().to_string(),
            )
        });
        p.into_iter()
            .map(|(_node_id, (last_seen, identity, payload))| {
                (PresenceFlag::from_instant(last_seen), last_seen, identity, payload)
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ChatPresenceData<T: ChatMessageType> {
    map: BTreeMap<NodeId, (Instant, NodeIdentity, T::P)>,
}
impl<T: ChatMessageType> Default for ChatPresenceData<T> {
    fn default() -> Self {
        Self { map: BTreeMap::new() }
    }
}

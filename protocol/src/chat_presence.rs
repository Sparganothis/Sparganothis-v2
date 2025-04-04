use iroh::NodeId;
use n0_future::time::Instant;
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::{Notify, RwLock};

use crate::{
    _const::{PRESENCE_EXPIRATION, PRESENCE_IDLE},
    signed_message::IChatRoomType,
    user_identity::NodeIdentity,
};

#[derive(Clone, Debug)]
pub struct ChatPresence<T: IChatRoomType> {
    presence: Arc<RwLock<ChatPresenceData<T>>>,
    notify: Arc<Notify>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum PresenceFlag {
    ACTIVE,
    IDLE,
    EXPIRED,
    UNCONFIRMED,
}

impl PresenceFlag {
    pub fn from_instant(instant: Instant) -> Self {
        let duration = instant.elapsed();
        if duration < PRESENCE_IDLE {
            Self::ACTIVE
        } else if duration < PRESENCE_EXPIRATION {
            Self::IDLE
        } else {
            Self::EXPIRED
        }
    }
}
pub type PresenceList<T> = Vec<PresenceListItem<T>>;

#[derive(Clone, Debug, PartialEq)]
pub struct PresenceListItem<T: IChatRoomType> {
    pub presence_flag: PresenceFlag,
    pub last_seen: Instant,
    pub identity: NodeIdentity,
    pub payload: Option<<T as IChatRoomType>::P>,
    pub rtt: Option<u16>,
}

impl<T: IChatRoomType> ChatPresence<T> {
    pub fn new() -> Self {
        Self {
            presence: Arc::new(RwLock::new(ChatPresenceData::default())),
            notify: Arc::new(Notify::new()),
        }
    }
    pub fn notified(&self) -> tokio::sync::futures::Notified<'_> {
        self.notify.notified()
    }
    /// returns true if the presence was added to list
    pub async fn add_presence(
        &self,
        identity: &NodeIdentity,
        payload: &Option<T::P>,
    ) -> bool {
        let identity = identity.clone();
        let now = Instant::now();
        let mut w = self.presence.write().await;
        let old_value = w.clone();
        let old_ping = w
            .map
            .get(&identity.node_id().clone())
            .map(|(_, _, _, rtt)| rtt.clone())
            .unwrap_or(None);
        let was_added = w
            .map
            .insert(
                identity.node_id().clone(),
                (now, identity, payload.clone(), old_ping),
            )
            .is_none();
        w.map.retain(|_, (last_seen, _, _, _)| {
            now.duration_since(*last_seen) < PRESENCE_EXPIRATION
        });
        let new_value = w.clone();
        if old_value != new_value {
            self.notify.notify_waiters();
        }
        was_added
    }
    pub async fn update_ping(&self, identity: &NodeIdentity, rtt: u16) {
        let identity = identity.clone();
        let mut w = self.presence.write().await;
        let Some(entry) = w.map.get_mut(&identity.node_id().clone()) else {
            return;
        };
        entry.3 = Some(rtt);
    }
    pub async fn get_presence_list(&self) -> PresenceList<T> {
        let p_map = self.presence.read().await.map.clone();
        let p = p_map.clone();
        let mut p = p.into_iter().collect::<Vec<_>>();
        p.sort_by_key(|(_, (_k, _userid, _payload, _rtt))| {
            (
                _userid.user_id().to_string(),
                _userid.nickname().to_string(),
            )
        });
        let v: Vec<_> = p
            .into_iter()
            .map(|(_node_id, (last_seen, identity, payload, rtt))| {
                PresenceListItem {
                    presence_flag: PresenceFlag::from_instant(last_seen),
                    last_seen,
                    identity,
                    payload,
                    rtt,
                }
            })
            .collect();

        v
    }
    pub async fn remove_presence(&self, identity: &NodeIdentity) {
        let identity = identity.clone();
        let mut w = self.presence.write().await;
        if w.map.remove(&identity.node_id().clone()).is_some() {
            self.notify.notify_waiters();
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ChatPresenceData<T: IChatRoomType> {
    map: BTreeMap<NodeId, (Instant, NodeIdentity, Option<T::P>, Option<u16>)>,
}
impl<T: IChatRoomType> Default for ChatPresenceData<T> {
    fn default() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
}

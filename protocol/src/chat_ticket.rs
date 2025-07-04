use std::collections::BTreeSet;

pub use iroh::NodeId;
pub use iroh_gossip::proto::TopicId;
use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct ChatTicket {
    pub topic_id: TopicId,
    pub bootstrap: BTreeSet<NodeId>,
}

impl ChatTicket {
    pub fn new_str_bs(topic_id: &str, bs: BTreeSet<NodeId>) -> Self {
        let mut topic_bytes = [0; 32];
        let topic_str = topic_id.as_bytes().to_vec();
        // assert!(topic_str.len() <= 30);
        let len = 30.min(topic_str.len());
        topic_bytes[..len].copy_from_slice(&topic_str[..len]);
        Self {
            topic_id: TopicId::from_bytes(topic_bytes),
            bootstrap: bs,
        }
    }
}

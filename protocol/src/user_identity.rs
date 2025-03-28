use iroh::{PublicKey, SecretKey};
use matchbox_socket::PeerId;

#[derive(
    Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq,
)]
pub struct UserIdentity {
    user_id: PublicKey,
}

impl UserIdentity {
    pub fn nickname(&self) -> String {
        crate::_random_word::get_nickname_from_pubkey(self.user_id.clone())
    }
    pub fn user_id(&self) -> &PublicKey {
        &self.user_id
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserIdentitySecrets {
    _user_private_key: SecretKey,
    user_identity: UserIdentity,
}

impl PartialEq for UserIdentitySecrets {
    fn eq(&self, other: &Self) -> bool {
        self.user_identity == other.user_identity
            && self._user_private_key.public()
                == other._user_private_key.public()
    }
}

impl UserIdentitySecrets {
    pub fn user_identity(&self) -> &UserIdentity {
        &self.user_identity
    }
    pub fn secret_key(&self) -> &SecretKey {
        &self._user_private_key
    }
    pub fn generate() -> Self {
        let _user_private_key = SecretKey::generate(rand::thread_rng());
        let user_id = _user_private_key.public();
        let user_identity = UserIdentity { user_id };
        Self {
            _user_private_key,
            user_identity,
        }
    }
}

#[derive(
    Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq,
)]
pub struct NodeIdentity {
    user_identity: UserIdentity,
    node_id: PublicKey,
    matchbox_id: PeerId,
    bootstrap_idx: Option<u32>,
}

impl NodeIdentity {
    pub fn nickname(&self) -> String {
        if let Some(bootstrap_idx) = self.bootstrap_idx {
            format!(
                "{} (bootstrap #{})",
                self.user_identity.nickname(),
                bootstrap_idx
            )
        } else {
            self.user_identity.nickname().to_string()
        }
    }
    pub fn user_id(&self) -> &PublicKey {
        &self.user_identity.user_id()
    }
    pub fn node_id(&self) -> &PublicKey {
        &self.node_id
    }
    pub fn user_identity(&self) -> &UserIdentity {
        &self.user_identity
    }
    pub fn bootstrap_idx(&self) -> Option<u32> {
        self.bootstrap_idx
    }
    pub fn matchbox_id(&self) -> &PeerId {
        &self.matchbox_id
    }
    pub fn new(
        user_identity: UserIdentity,
        node_id: PublicKey,
        matchbox_id: PeerId,
        bootstrap_idx: Option<u32>,
    ) -> Self {
        Self {
            user_identity,
            node_id,
            matchbox_id,
            bootstrap_idx,
        }
    }
}

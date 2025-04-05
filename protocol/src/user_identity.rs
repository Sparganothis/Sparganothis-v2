use iroh::{PublicKey, SecretKey};

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
    pub fn html_color(&self) -> String {
        let color = self.rgb_color();
        format!("rgb({},{},{})", color.0, color.1, color.2)
    }
    pub fn rgb_color(&self) -> (u8, u8, u8) {
        let pubkey_bytes = self.user_id.as_bytes();
        let mut color = [0_u8; 3];
        for i in 0..32 {
            let k = i % 3 as usize;
            color[k] = color[k] ^ pubkey_bytes[i];
        }
        (color[0], color[1], color[2])
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
    pub fn html_color(&self) -> String {
        self.user_identity.html_color()
    }
    pub fn rgb_color(&self) -> (u8, u8, u8) {
        self.user_identity.rgb_color()
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

    pub fn new(
        user_identity: UserIdentity,
        node_id: PublicKey,
        bootstrap_idx: Option<u32>,
    ) -> Self {
        Self {
            user_identity,
            node_id,
            bootstrap_idx,
        }
    }
}

//! DECLARATIONS OF API METHODS WITH SERVER IMPLEMENTATIONS

use game::api::game_match::GameMatch;

use crate::{declare_api_method, user_identity::NodeIdentity};

declare_api_method!(LoginApiMethod, (), ());

declare_api_method!(SendNewMatch, (GameMatch<NodeIdentity>, ), ());
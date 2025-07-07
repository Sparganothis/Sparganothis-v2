//! DECLARATIONS OF API METHODS WITH SERVER IMPLEMENTATIONS

use game::{api::game_match::GameMatch, tet::GameState};
use serde::{Deserialize, Serialize};

use crate::{declare_api_method, user_identity::NodeIdentity};

declare_api_method!(LoginApiMethod, (), ());

declare_api_method!(SendNewMatch, (GameMatch<NodeIdentity>,), ());

declare_api_method!(SendNewGameState, (GameMatch<NodeIdentity>, GameState), ());

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MatchRow2 {
    pub game_type: String,
    pub start_time: i64,
    pub user_ids: Vec<String>,
    pub game_seed: String,
    pub match_id: String,
    pub data_version: i64,
    pub match_info: Option<GameMatch<NodeIdentity>>,
}

declare_api_method!(GetReplayMatchList, (), Vec<MatchRow2>);
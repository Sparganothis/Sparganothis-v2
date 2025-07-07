//! DECLARATIONS OF API METHODS WITH SERVER IMPLEMENTATIONS

use game::{api::game_match::GameMatch, tet::GameState};

use crate::{declare_api_method, user_identity::NodeIdentity};

declare_api_method!(LoginApiMethod, (), ());

declare_api_method!(SendNewMatch, (GameMatch<NodeIdentity>,), ());

declare_api_method!(SendNewGameState, (GameMatch<NodeIdentity>, GameState), ());

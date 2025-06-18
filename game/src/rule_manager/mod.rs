use std::future::Future;

use crate::tet::GameState;

#[async_trait::async_trait]
pub trait RuleManager {
    // Accepts changed game state (from our or another rule manager)
    // and then outputs suggested next state (if the rule thinks something else should happen).
    // Rule can wait until condition happens.
    // When new game state is accepted, previous call is dropped (async future context is cancelled) and this function is re-executed.
    // Errors from this manager should be logged in chat.
    async fn accept_state(&self, state: GameState)
        -> anyhow::Result<Option<GameState>>;
}

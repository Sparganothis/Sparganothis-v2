use std::{future::Future, sync::Arc};

use async_stream::stream;
use futures_util::StreamExt;
use n0_future::{task::AbortOnDropHandle, Stream};
use rand::{rng, Rng};
use tokio::sync::{Notify, RwLock};

use crate::{
    rule_manager::RuleManager,
    tet::{GameSeed, GameState},
};

#[derive(Clone)]
pub struct GameStateManager {
    state: Arc<RwLock<GameState>>,
    notify: Arc<Notify>,
    rule_managers: Vec<(String, Arc<dyn RuleManager + 'static + Send + Sync>)>,
    loops: Vec<Arc<AbortOnDropHandle<anyhow::Result<()>>>>,
    obj_id: u64,
}

impl PartialEq for GameStateManager {
    fn eq(&self, other: &Self) -> bool {
        self.obj_id == other.obj_id
    }
}

impl GameStateManager {
    pub async fn get_state(&self) -> GameState {
        self.state.read().await.clone()
    }
    pub fn new(game_seed: &GameSeed, start_time: i64) -> Self {
        let state = GameState::new(game_seed, start_time);
        let id: u64 = (&mut rng()).random();
        tracing::info!("INIT GAME MANAGER {id}");

        Self {
            state: Arc::new(RwLock::new(state)),
            notify: Arc::new(Notify::new()),
            rule_managers: vec![],
            loops: vec![],
            obj_id: id,
        }
    }

    pub fn add_rule(
        &mut self,
        name: &str,
        rule: Arc<dyn RuleManager + 'static + Send + Sync>,
    ) {
        tracing::info!("GAME MANAGER ADD RULE.");
        self.rule_managers.push((name.to_string(), rule));
    }
    pub fn add_loop<F: Future<Output = anyhow::Result<()>> + Send + 'static>(
        &mut self,
        _loop: F,
    ) {
        tracing::info!("GAME MANAGER ADD LOOP.");
        self.loops
            .push(Arc::new(AbortOnDropHandle::new(n0_future::task::spawn({
                async move {
                    let r = _loop.await;
                    if let Err(ref e) = r {
                        tracing::error!("GameStateManager(): error in loop: {:#?} ", e)
                    }
                    tracing::warn!("GameManager(): LOOP EXITED WITH NO ERROR!!");
                    r
                }
            }))));
    }

    pub async fn main_loop(&self) -> anyhow::Result<()> {
        let mut current_state = self.get_state().await;
        self.notify.notify_waiters();
        tracing::info!("GameManager(): main_loop() started");

        while !current_state.game_over() {
            let mut fut = n0_future::FuturesUnordered::new();
            let state2 = current_state;
            for manager in self.rule_managers.iter() {
                let manager = manager.clone();
                let next = async move {
                    (manager.0.clone(), manager.1.accept_state(state2).await)
                };
                fut.push(next);
            }
            while let Some((rule_name, result)) = fut.next().await {
                match result {
                    Ok(Some(result)) => {
                        drop(fut);

                        current_state = result;
                        self._set_state_and_notify(result).await;
                        break;
                    }
                    Ok(None) => {
                        // do nothing
                    }
                    Err(err) => {
                        tracing::error!("rule {rule_name} returned error: {:#?}", err);
                    }
                }
            }
        }
        tracing::warn!("GameStateManager -- main loop EXIT.");
        Ok(())
    }

    async fn _set_state_and_notify(&self, new_state: GameState) {
        {
            *self.state.write().await = new_state;
        }
        self.notify.notify_one();
        self.notify.notify_waiters();
    }

    pub fn read_state_stream(&self) -> impl Stream<Item = GameState> + Send + 'static {
        let state_arc = self.state.clone();
        let notify_arc = self.notify.clone();

        stream! {
            let mut state = {state_arc.read().await.clone()};
            tracing::info!("StateManager state_stream() --- init");
            yield state;
            loop {
                let _x = notify_arc.notified().await;
                let new_state =  {state_arc.read().await.clone()};
                if new_state != state {
                    state = new_state;
                    yield state;
                }
            }
        }
    }
}

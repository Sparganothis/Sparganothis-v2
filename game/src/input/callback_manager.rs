use std::{collections::BTreeMap, sync::Arc, time::Duration};

use super::events::GameInputEvent;
use super::input_manager::{CallbackMoveType, CallbackTicket, UserEvent};
use crate::input::input_manager::GameInputManager;
use crate::settings::GameSettings;
use crate::tet::GameState;
use crate::{tet::TetAction, timestamp::get_timestamp_now_ms};
use async_stream::stream;
use futures_channel::mpsc::{unbounded, UnboundedReceiver};
use futures_core::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::{pin_mut, FutureExt};
use n0_future::task::AbortOnDropHandle;
use tokio::sync::Mutex;
use tokio::sync::{futures::Notified, Notify, RwLock};

#[derive(Debug, Clone)]
struct CallbackManager {
    inner: Arc<RwLock<CallbackManagerInner>>,
    _notify: Arc<Notify>,
}

impl CallbackManager {
    // pub fn
    pub fn new2() -> Self {
        let notify = Arc::new(Notify::new());
        let _notify = notify.clone();
        let mut inner = CallbackManagerInner {
            events: BTreeMap::new(),
            notify,
        };
        inner.set_cb(CallbackMoveType::AutoSoftDrop, Duration::from_millis(100));
        Self {
            inner: Arc::new(RwLock::new(inner)),
            _notify,
        }
    }
    pub fn notified(&self) -> Notified<'_> {
        self._notify.notified()
    }
    pub async fn accept_user_event(&self, user_event: UserEvent) -> Option<TetAction> {
        let mut g = self.inner.write().await;
        g.accept_user_event(user_event)
    }

    // private fn

    async fn get_events(&self) -> BTreeMap<CallbackMoveType, i64> {
        let e = {
            let g = self.inner.read().await;
            g.events.clone()
        };
        e
    }

    pub async fn get_sleep_duration_ms(&self) -> (i64, Vec<CallbackMoveType>) {
        let events = self.get_events().await;

        let now = get_timestamp_now_ms();
        let mut min_delay = 10000;
        let mut v = vec![];
        for (event, timestamp_expired) in events {
            if timestamp_expired <= now {
                v.push(event);
            } else {
                let new_delay = timestamp_expired - now;
                if new_delay < min_delay {
                    min_delay = new_delay;
                }
            }
        }
        (min_delay, v)
    }

    pub async fn main_loop(
        &self,
        mut _r: UnboundedReceiver<(GameState, GameInputEvent)>,
        settings: Arc<RwLock<GameSettings>>,
    ) -> impl Stream<Item = TetAction> {
        let mut input_manager = GameInputManager::new();
        let callback_manager = self.clone();
        stream! {
            pin_mut!(_r);
            loop {
                let game_settings = {settings.read().await.clone()};
                let (mut duration_ms, items) =
                    callback_manager.get_sleep_duration_ms().await;
                for _move in items {
                    let x = input_manager
                        .callback_after_wait(_move, game_settings);
                    let y = callback_manager.accept_user_event(x).await;
                    if let Some(action) = y {
                        yield action;
                    }
                }
                duration_ms = duration_ms.clamp(1, 10000);
                if duration_ms > 8 {
                    duration_ms -= 8;
                }
                if duration_ms > 32 {
                    duration_ms -= 16;
                }
                if duration_ms > 32 {
                    duration_ms /= 2;
                }
                let duration =
                    std::time::Duration::from_millis(duration_ms as u64);
                // let t0 = get_timestamp_now_ms();

                tokio::select! {
                    kbd_event = _r.next().fuse() => {
                        let Some((state, kbd_event)) = kbd_event else {
                            tracing::warn!("ticket manger loop end: coro end");
                            break;
                        };

                        let settings =  {settings.read().await.clone()};
                        let event = input_manager
                            .on_user_keyboard_event(kbd_event, settings, &state);
                        let y = callback_manager.accept_user_event(event).await;
                        if let Some(action) = y {
                            yield action;
                        }
                        continue;
                    }
                    _not = callback_manager.notified().fuse() => {
                        continue;
                    }

                    _sl = n0_future::time::sleep(duration).fuse() => {
                        // let t1 = get_timestamp_now_ms();
                        // let dt = t1 - t0;
                        // let diff = dt as i32 - duration_ms as i32;
                        // tracing::info!("callbackmanager loop sleep diff: {}", diff);
                        continue;
                    }
                }
            }
        }
        .boxed()
    }
}

#[derive(Clone, Debug)]
struct CallbackManagerInner {
    events: BTreeMap<CallbackMoveType, i64>,
    notify: Arc<Notify>,
}

impl CallbackManagerInner {
    fn accept_user_event(&mut self, user_event: UserEvent) -> Option<TetAction> {
        let action = user_event.action;

        for ticket in user_event.callback_tickets {
            self.accept_ticket(ticket);
        }

        self.notify.notify_one();

        action
    }

    fn accept_ticket(&mut self, ticket: CallbackTicket) {
        match ticket.request_type {
            super::input_manager::CallbackRequestType::SetCallback(duration) => {
                self.set_cb(ticket.move_type, duration);
            }
            super::input_manager::CallbackRequestType::DropCallback => {
                self.drop_cb(ticket.move_type);
            }
        }
    }

    fn drop_cb(&mut self, move_type: CallbackMoveType) {
        self.events.remove(&move_type);
        self.notify.notify_waiters();
    }
    fn set_cb(&mut self, move_type: CallbackMoveType, duration: Duration) {
        let now = get_timestamp_now_ms();
        self.events
            .insert(move_type, now + duration.as_millis() as i64);
        self.notify.notify_waiters();
    }
}

use crate::rule_manager::RuleManager;

#[derive(Debug)]
pub struct InputCallbackManagerRule {
    cb_manager: CallbackManager,
    main_loop: AbortOnDropHandle<anyhow::Result<()>>,
    stream_loop: AbortOnDropHandle<anyhow::Result<()>>,
    action_receiver: Mutex<UnboundedReceiver<TetAction>>,
}

impl InputCallbackManagerRule {
    pub fn new(
        mut input_stream: UnboundedReceiver<GameInputEvent>,
        state_stream: impl Stream<Item = GameState> + Send + 'static,
        settings: Arc<RwLock<GameSettings>>,
    ) -> Self {
        let cb_manager = CallbackManager::new2();

        let (pair_tx, pair_rx) = unbounded();
        let (action_tx, action_rx) = unbounded();

        let stream_loop = AbortOnDropHandle::new(n0_future::task::spawn(async move {
            pin_mut!(state_stream);
            let Some(mut state) = state_stream.next().await else {
                anyhow::bail!("no initial state.");
            };

            loop {
                tokio::select! {
                    kbd_event = input_stream.next().fuse() => {
                        let Some(kbd_event) = kbd_event else {
                            anyhow::bail!("no more kbd events");
                        };
                        pair_tx.unbounded_send((state, kbd_event))?;
                    }
                    new_state = state_stream.next().fuse() => {
                        let Some(new_state) = new_state else {
                            anyhow::bail!("no more states.");
                        };
                        state = new_state ;
                    }
                }
            }
        }));

        Self {
            cb_manager: cb_manager.clone(),
            main_loop: AbortOnDropHandle::new(n0_future::task::spawn(async move {
                let mut s = cb_manager.main_loop(pair_rx, settings).await;
                while let Some(x) = s.next().await {
                    action_tx.unbounded_send(x)?;
                }

                anyhow::Ok(())
            })),
            stream_loop,
            action_receiver: Mutex::new(action_rx),
        }
    }
}

#[async_trait::async_trait]
impl RuleManager for InputCallbackManagerRule {
    async fn accept_state(
        &self,
        _state: GameState,
    ) -> anyhow::Result<Option<GameState>> {
        let mut recv = self.action_receiver.lock().await;
        let Some(next_action) = recv.next().fuse().await else {
            anyhow::bail!("no next action");
        };
        let next_state = _state.try_action(next_action, get_timestamp_now_ms())?;
        Ok(Some(next_state))
    }
}

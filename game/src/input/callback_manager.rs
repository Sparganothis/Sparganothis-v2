use std::{collections::BTreeMap, sync::Arc, time::Duration};

use super::events::GameInputEvent;
use super::input_manager::{CallbackMoveType, CallbackTicket, UserEvent};
use crate::input::input_manager::GameInputManager;
use crate::settings::GameSettings;
use crate::{tet::TetAction, timestamp::get_timestamp_now_ms};
use async_stream::stream;
use futures_channel::mpsc::UnboundedReceiver;
use futures_core::stream::Stream;
use futures_util::stream::StreamExt;
use futures_util::{pin_mut, FutureExt};
use tokio::sync::{futures::Notified, Mutex, Notify};

#[derive(Debug, Clone)]
pub struct CallbackManager {
    inner: Arc<Mutex<CallbackManagerInner>>,
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
            inner: Arc::new(Mutex::new(inner)),
            _notify,
        }
    }
    pub fn notified(&self) -> Notified<'_> {
        self._notify.notified()
    }
    pub async fn accept_user_event(&self, user_event: UserEvent) -> Option<TetAction> {
        let mut g = self.inner.lock().await;
        g.accept_user_event(user_event)
    }

    // private fn

    async fn get_events(&self) -> BTreeMap<CallbackMoveType, i64> {
        let e = {
            let g = self.inner.lock().await;
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
        mut _r: UnboundedReceiver<GameInputEvent>,
        settings: Arc<Mutex<GameSettings>>,
    ) -> impl Stream<Item = TetAction> {
        let input_manager = Arc::new(Mutex::new(GameInputManager::new()));
        let callback_manager = self.clone();
        stream! {
            pin_mut!(_r);
            loop {
                let game_settings = {settings.lock().await.clone()};
                let (duration_ms, items) =
                    callback_manager.get_sleep_duration_ms().await;
                for _move in items {
                    let x = {input_manager
                        .lock().await
                        .callback_after_wait(_move, game_settings)};
                    let y = callback_manager.accept_user_event(x).await;
                    if let Some(action) = y {
                        yield action;
                    }
                }
                let duration =
                    std::time::Duration::from_millis(duration_ms as u64);

                tokio::select! {
                    kbd_event = _r.next().fuse() => {
                        let Some(kbd_event) = kbd_event else {
                            tracing::warn!("ticket manger loop end: coro end");
                            break;
                        };

                        let settings =  {settings.lock().await.clone()};
                        let event = {input_manager
                            .lock().await
                            .on_user_keyboard_event(kbd_event, settings)};
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

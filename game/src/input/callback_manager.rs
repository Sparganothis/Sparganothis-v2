use std::{collections::BTreeMap, sync::Arc, time::Duration};

use super::input_manager::{CallbackMoveType, CallbackTicket, UserEvent};
use crate::{tet::TetAction, timestamp::get_timestamp_now_ms};
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

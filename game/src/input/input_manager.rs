use std::collections::BTreeSet;
use std::time::Duration;

use crate::input::events::GameInputEvent;
use crate::tet::TetAction;

#[derive(Clone, Debug)]
pub struct GameInputManager {
    new_held: BTreeSet<TetAction>,
    old_held: BTreeSet<TetAction>,
    move_auto_repeat_first_ms: u16,
    move_auto_repeat_after_ms: u16,
    move_auto_soft_drop_ms: u16,
}

impl GameInputManager {
    pub fn new() -> Self {
        Self {
            move_auto_repeat_first_ms: 150,
            move_auto_repeat_after_ms: 33,
            move_auto_soft_drop_ms: 666,
            new_held: BTreeSet::new(),
            old_held: BTreeSet::new(),
        }
    }

    pub fn on_user_keyboard_event(
        &mut self,
        user_keyboard_event: GameInputEvent,
    ) -> UserEvent {
        let GameInputEvent {
            key,
            event,
            ts: _ts,
        } = user_keyboard_event;
        let Some(action) = key.to_game_action() else {
            return UserEvent::empty();
        };
        match event {
            super::events::GameInputEventType::KeyDown => {
                self.new_held.insert(action);
            }
            super::events::GameInputEventType::KeyUp => {
                self.new_held.remove(&action);
            }
        }
        let new_down: BTreeSet<TetAction> =
            self.new_held.difference(&self.old_held).cloned().collect();
        let _new_up: BTreeSet<TetAction> =
            self.old_held.difference(&self.new_held).cloned().collect();

        self.old_held = self.new_held.clone();

        let mut cb = vec![];
        for key_up in _new_up {
            let move_type = match key_up {
                TetAction::MoveLeft => CallbackMoveType::AutoMoveLeft,
                TetAction::MoveRight => CallbackMoveType::AutoMoveRight,
                TetAction::SoftDrop => CallbackMoveType::AutoMoveDown,
                _ => continue,
            };
            cb.push(CallbackTicket {
                request_type: CallbackRequestType::DropCallback,
                move_type,
            })
        }
        for key_down in new_down.iter().cloned() {
            let move_type = match key_down {
                TetAction::MoveLeft => CallbackMoveType::AutoMoveLeft,
                TetAction::MoveRight => CallbackMoveType::AutoMoveRight,
                TetAction::SoftDrop => CallbackMoveType::AutoMoveDown,
                _ => continue,
            };
            cb.push(CallbackTicket {
                request_type: CallbackRequestType::SetCallback(Duration::from_millis(
                    self.move_auto_repeat_first_ms as u64,
                )),
                move_type,
            })
        }
        cb.push(CallbackTicket {
            request_type: CallbackRequestType::SetCallback(Duration::from_millis(
                self.move_auto_soft_drop_ms as u64,
            )),
            move_type: CallbackMoveType::AutoSoftDrop,
        });

        let event = UserEvent {
            callback_tickets: cb,
            action: new_down.first().cloned(),
        };

        event
    }

    pub fn callback_after_wait(
        &mut self,
        callback_move_type: CallbackMoveType,
    ) -> UserEvent {
        let action = match callback_move_type {
            CallbackMoveType::AutoMoveDown => TetAction::SoftDrop,
            CallbackMoveType::AutoMoveLeft => TetAction::MoveLeft,
            CallbackMoveType::AutoMoveRight => TetAction::MoveRight,
            CallbackMoveType::AutoSoftDrop => TetAction::SoftDrop,
        };
        let mut cb = vec![];

        let cb_duration = match callback_move_type {
            // TODO: if game's next soft drop will lock, put a longer timeout here
            CallbackMoveType::AutoSoftDrop => self.move_auto_soft_drop_ms,
            CallbackMoveType::AutoMoveDown => self.move_auto_repeat_after_ms,
            CallbackMoveType::AutoMoveLeft => self.move_auto_repeat_after_ms,
            CallbackMoveType::AutoMoveRight => self.move_auto_repeat_after_ms,
        };
        cb.push(CallbackTicket {
            request_type: CallbackRequestType::SetCallback(Duration::from_millis(
                cb_duration as u64,
            )),
            move_type: callback_move_type,
        });

        UserEvent {
            callback_tickets: cb,
            action: Some(action),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserEvent {
    // set if game should get action
    pub action: Option<TetAction>,
    // if set, call in future using this ticket
    pub callback_tickets: Vec<CallbackTicket>,
}

impl UserEvent {
    fn empty() -> Self {
        Self {
            action: None,
            callback_tickets: vec![],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CallbackTicket {
    pub request_type: CallbackRequestType,
    pub move_type: CallbackMoveType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallbackMoveType {
    AutoSoftDrop,
    AutoMoveLeft,
    AutoMoveRight,
    AutoMoveDown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallbackRequestType {
    SetCallback(Duration),
    DropCallback,
}

use std::collections::BTreeSet;
use std::time::Duration;

use crate::input::events::GameInputEvent;
use crate::settings::GameSettings;
use crate::tet::{GameState, TetAction};

#[derive(Clone, Debug)]
pub struct GameInputManager {
    new_held: BTreeSet<TetAction>,
    old_held: BTreeSet<TetAction>,
}

impl GameInputManager {
    pub fn new() -> Self {
        Self {
            new_held: BTreeSet::new(),
            old_held: BTreeSet::new(),
        }
    }

    pub fn on_user_keyboard_event(
        &mut self,
        user_keyboard_event: GameInputEvent,
        game_settings: GameSettings,
        game_state: &GameState,
    ) -> UserEvent {
        let GameInputEvent {
            key,
            event,
            ts: _ts,
        } = user_keyboard_event;

        let Some(action) = key.to_game_action() else {
            return UserEvent::empty();
        };

        if game_state.game_over() {
            return UserEvent::empty();
        }

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
                TetAction::MoveLeft => CallbackMoveType::RepeatMoveLeft,
                TetAction::MoveRight => CallbackMoveType::RepeatMoveRight,
                TetAction::UserSoftDrop => CallbackMoveType::RepeatMoveDown,
                _ => continue,
            };
            cb.push(CallbackTicket {
                request_type: CallbackRequestType::DropCallback,
                move_type,
            })
        }
        for key_down in new_down.iter().cloned() {
            // if invalid move, do not apply
            let move_type = match key_down {
                TetAction::MoveLeft => CallbackMoveType::RepeatMoveLeft,
                TetAction::MoveRight => CallbackMoveType::RepeatMoveRight,
                TetAction::UserSoftDrop => CallbackMoveType::RepeatMoveDown,
                _ => continue,
            };
            cb.push(CallbackTicket {
                request_type: CallbackRequestType::SetCallback(
                    game_settings.input.autorepeat_delay_initial,
                ),
                move_type,
            })
        }
        // TODO: smartter refresh interval to avoid floating
        // if action == TetAction::HardDrop {
        if action != TetAction::AutoSoftDrop && action != TetAction::UserSoftDrop {
            if game_state.try_action(action, 0).is_ok() {
                cb.push(CallbackTicket {
                    request_type: CallbackRequestType::SetCallback(
                        game_settings.game.auto_softdrop_interval,
                    ),
                    move_type: CallbackMoveType::AutoSoftDrop,
                });
            }
        }

        let event = UserEvent {
            callback_tickets: cb,
            action: new_down.first().cloned(),
        };

        event
    }

    pub fn callback_after_wait(
        &mut self,
        callback_move_type: CallbackMoveType,
        game_settings: GameSettings,
    ) -> UserEvent {
        let action = match callback_move_type {
            CallbackMoveType::RepeatMoveDown => TetAction::UserSoftDrop,
            CallbackMoveType::RepeatMoveLeft => TetAction::MoveLeft,
            CallbackMoveType::RepeatMoveRight => TetAction::MoveRight,
            CallbackMoveType::AutoSoftDrop => TetAction::AutoSoftDrop,
        };
        let mut cb = vec![];

        let cb_duration = match callback_move_type {
            // TODO: if game's next soft drop will lock, put a longer timeout here
            CallbackMoveType::AutoSoftDrop => game_settings.game.auto_softdrop_interval,
            CallbackMoveType::RepeatMoveDown => {
                game_settings.input.autorepeat_delay_after
            }
            CallbackMoveType::RepeatMoveLeft => {
                game_settings.input.autorepeat_delay_after
            }
            CallbackMoveType::RepeatMoveRight => {
                game_settings.input.autorepeat_delay_after
            }
        };
        cb.push(CallbackTicket {
            request_type: CallbackRequestType::SetCallback(cb_duration),
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
    RepeatMoveLeft,
    RepeatMoveRight,
    RepeatMoveDown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallbackRequestType {
    SetCallback(Duration),
    DropCallback,
}

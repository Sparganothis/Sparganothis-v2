use std::collections::BTreeSet;
use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::tet::TetAction;
use crate::input::events::GameInputEvent;

#[derive(Clone, Debug)]
pub struct GameInputManager {
    new_held: BTreeSet<TetAction>,
    old_held: BTreeSet<TetAction>,
    repeat_ms: u16,
}


impl GameInputManager {
    pub fn new() -> Self {
        Self {
            repeat_ms: 250,
            new_held: BTreeSet::new(),
            old_held: BTreeSet::new(),
        }
    }

    pub fn on_user_event(&mut self, user_event: GameInputEvent) -> Option<(TetAction, Option<std::time::Duration>)> {
        let GameInputEvent { key, event, ts } = user_event;
        let Some(action) = key.to_game_action() else {
            return None;
        };
        match event {
            super::events::GameInputEventType::KeyDown => {
                self.new_held.insert(action);

            },
            super::events::GameInputEventType::KeyUp => {
                self.new_held.remove(&action);
            },
        }
        let new_down: BTreeSet<TetAction> = self.new_held.difference(&self.old_held).cloned().collect();
        let _new_up : BTreeSet<TetAction> = self.old_held.difference(&self.new_held).cloned().collect();

        self.old_held = self.new_held.clone();

        let cb = action == TetAction::MoveLeft || 
        action == TetAction::MoveRight ||
        action == TetAction::SoftDrop; 

        let cb = if cb {
            Some(Duration::from_millis(self.repeat_ms as u64))
        } else {
            None
        };
        new_down.first().cloned().map(|c| (c, cb))
    }

    pub fn after_wait(&mut self, action: TetAction) -> Option<(TetAction, Option<std::time::Duration>)> {
        let cb = action == TetAction::MoveLeft || 
        action == TetAction::MoveRight ||
        action == TetAction::SoftDrop; 
        if !cb {
            return None;
        }
        Some((action, Some(Duration::from_millis(self.repeat_ms as u64))))
    }

}
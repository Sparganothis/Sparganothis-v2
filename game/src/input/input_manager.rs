use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use crate::tet::TetAction;
use crate::input::events::GameInputEvent;

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
    pub fn on_user_event(&mut self, user_event: GameInputEvent) -> Option<TetAction> {
        let GameInputEvent { key, event, ts } = user_event;
        let action = key.to_game_action()?;
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

        new_down.first().cloned()
    }
}
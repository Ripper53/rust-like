use bevy::prelude::Entity;
use crate::PlayerState;

#[derive(Default)]
pub struct Dialogue {
    pub entity: Option<Entity>,
    pub text: String,
    pub options: Vec<(String, DialogueOption)>,
}
impl Dialogue {
    pub fn activate(&mut self, current_player_state: PlayerState, entity: Entity, text: String, options: Vec<(String, DialogueOption)>) -> PlayerState {
        if !matches!(current_player_state, PlayerState::None) { return current_player_state; }
        self.entity = Some(entity);
        self.text = text;
        self.options = options;
        PlayerState::Dialogue
    }
    pub fn select(&mut self, current_player_state: PlayerState, active: usize) -> PlayerState {
        if !matches!(current_player_state, PlayerState::Dialogue) { return current_player_state; }
        if let Some(entity) = self.entity {
            self.entity = None;
            let option = self.options[active].1.clone();
            option.execute(self, PlayerState::None, entity);
        }
        PlayerState::None
    }
}

#[derive(Clone)]
pub enum DialogueOption {
    /// Discontinue dialogue.
    Leave,
    /// Get more info, continue dialogue.
    Info(String, Vec<(String, DialogueOption)>),
}
impl DialogueOption {
    fn execute(&self, dialogue: &mut Dialogue, player_state: PlayerState, entity: Entity) {
        match self {
            DialogueOption::Leave => {},
            DialogueOption::Info(info, options) => {
                dialogue.activate(player_state, entity, info.to_owned(), options.to_owned());
            },
        }
    }
}

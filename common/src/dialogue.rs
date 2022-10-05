use crate::PlayerState;

#[derive(Default)]
pub struct Dialogue {
    pub text: String,
    pub options: Vec<(String, DialogueOption)>,
    /// Active option index.
    pub active: usize,
}
impl Dialogue {
    pub fn activate(&mut self, current_player_state: PlayerState, text: String, options: Vec<(String, DialogueOption)>) -> PlayerState {
        if !matches!(current_player_state, PlayerState::None) { return current_player_state; }
        self.text = text;
        self.options = options;
        self.active = 0;
        PlayerState::Dialogue
    }
    pub fn increment(&mut self) {
        if self.active != self.options.len() - 1 {
            self.active += 1;
        }
    }
    pub fn decrement(&mut self) {
        if self.active != 0 {
            self.active -= 1;
        }
    }
    pub fn select(&mut self, current_player_state: PlayerState) -> PlayerState {
        if !matches!(current_player_state, PlayerState::Dialogue) { return current_player_state; }
        let option = self.options[self.active].1.clone();
        option.execute(self, PlayerState::None);
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
    fn execute(&self, dialogue: &mut Dialogue, player_state: PlayerState) {
        match self {
            DialogueOption::Leave => {},
            DialogueOption::Info(info, options) => {
                dialogue.activate(player_state, info.to_owned(), options.to_owned());
            },
        }
    }
}

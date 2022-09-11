#[derive(Default)]
pub struct Dialogue {
    pub in_conversation: bool,
    pub text: String,
    pub options: Vec<(String, DialogueOption)>,
    /// Active option index.
    pub active: usize,
}
impl Dialogue {
    pub fn activate(&mut self, text: String, options: Vec<(String, DialogueOption)>) {
        self.text = text;
        self.options = options;
        self.active = 0;
        self.in_conversation = true;
    }
    pub fn increment(&mut self) {
        if self.active != self.options.len() {
            self.active += 1;
        }
    }
    pub fn decrement(&mut self) {
        if self.active != 0 {
            self.active -= 1;
        }
    }
    pub fn select(&mut self) {
        self.in_conversation = false;
        let option = &self.options[self.active].1;
    }
}

pub enum DialogueOption {
    None,
}

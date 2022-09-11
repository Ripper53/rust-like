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
        if self.active != self.options.len() - 1 {
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
        let option = self.options[self.active].1.clone();
        option.execute(self);
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
    fn execute(&self, dialogue: &mut Dialogue) {
        match self {
            DialogueOption::Leave => {},
            DialogueOption::Info(info, options) => {
                dialogue.activate(info.to_owned(), options.to_owned());
            },
        }
    }
}

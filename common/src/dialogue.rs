pub struct Dialogue {
    pub text: String,
    pub options: Vec<(String, Option)>,
    /// Active option index.
    pub active: usize,
}

pub enum Option {

}

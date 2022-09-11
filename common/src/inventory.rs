#[derive(Debug)]
pub enum Item {

}
impl Item {
    pub fn get_name(&self) -> &'static str {
        match self {
            _ => "Bruh",
        }
    }
}

#[derive(Debug, Default)]
pub struct Inventory {
    pub items: Vec<Item>,
}

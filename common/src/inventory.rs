#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Food,
}
impl Item {
    pub fn get_name(&self) -> &'static str {
        match self {
            Self::Food => "Food",
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Inventory {
    pub items: Vec<Item>,
}

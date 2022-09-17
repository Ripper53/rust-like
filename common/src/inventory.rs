#[derive(Clone)]
pub enum Item {
    Food {
        info: ItemBasicInfo,
        heal: i32,
    }
}

#[derive(Clone)]
pub struct ItemBasicInfo {
    name: String,
    description: String,
}

impl Item {
    pub fn get_name(&self) -> String {
        match self {
            Item::Food { info, .. } => info.name.clone(),
        }
    }
    pub fn get_description(&self) -> String {
        match self {
            Item::Food { info, .. } => info.description.clone(),
        }
    }
    fn new_food(name: String, heal: i32) -> Self {
        Item::Food {
            info: ItemBasicInfo {
                name,
                description: format!("Heal for {heal}."),
            },
            heal,
        }
    }
    pub fn new_apple() -> Self {
        Self::new_food("Apple".to_string(), 1)
    }
    pub fn new_banana() -> Self {
        Self::new_food("Banana".to_string(), 2)
    }
}

impl PartialEq for &Box<Item> {
    fn eq(&self, other: &Self) -> bool {
        let left: *const Item = self.as_ref();
        let right: *const Item = other.as_ref();
        left == right
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[derive(Default)]
pub struct Inventory {
    pub items: Vec<Box<Item>>,
}

impl Inventory {
    pub fn get_index(&self, item: &Box<Item>) -> Option<usize> {
        for i in 0..self.items.len() {
            if &self.items[i] == item {
                return Some(i);
            }
        }
        None
    }
}

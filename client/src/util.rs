use common::inventory::Inventory;
use tui::{widgets::{ListItem, List, Block, Borders}, text::Text};

pub fn render_inventory<'a>(inventory: &'a Inventory, title: &'a str) -> List {
    let mut items = Vec::<ListItem>::with_capacity(inventory.items().len());
    for item in inventory.items() {
        items.push(ListItem::new(Text::raw(item.get_name())));
    }
    List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_symbol(">")
}

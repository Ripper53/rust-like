use std::{time::{Duration, Instant}, thread, sync::mpsc::Receiver};

use bevy::prelude::{App, ResMut, Query, With, CoreStage, State, Entity, World};
use common::{physics::*, character::{PlayerInput, MovementInput, PlayerTag, ActionHistory, Health}, dialogue::Dialogue, inventory::{Inventory, Equipment}, ActionInput, Scene, PlayerState, loot_menu::{LootMenu, transfer_item}};
use crossterm::{
    terminal::enable_raw_mode, event, execute,
};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Layout, Constraint},
    widgets::{Paragraph, Block, Borders, Tabs, List, ListItem, ListState},
    style::{Style, Modifier},
    text::{Spans, Span, Text}
};

use crate::{canvas::MapCanvas, util::render_inventory};

enum Event<I> {
    Input(I),
    Tick,
}

fn setup_terminal() -> Result<Receiver<Event<event::Event>>, Box<dyn std::error::Error>> {
    enable_raw_mode().expect("Running in raw mode.");

    // Input
    let (tx, rx) = std::sync::mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("Poll works.") {
                let key = event::read().expect("Can read events.");
                tx.send(Event::Input(key)).expect("Can send events.");
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    Ok(rx)
}

#[derive(Default, PartialEq, Eq)]
enum Menu {
    #[default]
    World,
    Inventory,
    Settings,
}
impl From<&Menu> for usize {
    fn from(input: &Menu) -> Self {
        match input {
            Menu::World => 0,
            Menu::Inventory => 1,
            Menu::Settings => 2,
        }
    }
}

#[derive(Default)]
pub struct CameraData {
    position: Position,
    inventory_selection: ListState,
}
fn update_camera_system(mut camera: ResMut<CameraData>, query: Query<&Position, With<PlayerTag>>) {
    for position in query.iter() {
        camera.position = position.clone();
    }
}

pub fn runner(mut app: App) {
    setup_game(&mut app).expect("setup_game");
}

#[derive(Default)]
struct Data {
    active_menu: Menu,
    active_option: MenuOption,
}
#[derive(Default)]
struct MenuOption {
    focus: Focus,
    index: usize,
}
#[derive(Default)]
enum Focus {
    Ours,
    #[default]
    Other,
}
impl MenuOption {
    fn increment(&mut self, count: usize) {
        if self.index != count - 1 {
            self.index += 1;
        }
    }
    fn decrement(&mut self) {
        if self.index != 0 {
            self.index -= 1;
        }
    }
    fn get_index(&self, count: usize) -> Option<usize> {
        if count == 0 {
            None
        } else {
            Some(self.index)
        }
    }
    /// Makes sure index is not greater than count!
    fn check(&mut self, count: usize) {
        if  self.index >= count {
            self.index = count - 1;
        }
    }
    fn check_from_focus(&mut self, world: &mut World) {
        match self.focus {
            Focus::Ours => {
                let mut player_inventory = world.query_filtered::<&Inventory, With<PlayerTag>>();
                let player_inventory = player_inventory.single(world);
                self.check(player_inventory.items().len());
            },
            Focus::Other => {
                let loot_menu = world.resource::<LootMenu>();
                if let Some(entity) = loot_menu.inventory {
                    if let Some(inventory) = world.entity(entity).get::<Inventory>() {
                        self.check(inventory.items().len());
                    }
                }
            },
        }
    }
}

fn setup_game(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    app
        .insert_resource(CameraData::default())
        .add_startup_system(update_camera_system)
        .add_system_to_stage(CoreStage::PostUpdate, update_camera_system);
    app.update();

    // Input
    let rx = setup_terminal()?;
    execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen).ok();
    let backend = CrosstermBackend::new(std::io::stdout());

    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["World", "Inventory", "Settings"];
    let mut data = Data::default();

    // Render
    loop {
        terminal.draw(|rect| {
            const MARGIN: u16 = 2;
            // Layout
            let player_state = app.world.resource::<PlayerState>();
            let top_layout = Layout::default()
                .direction(tui::layout::Direction::Horizontal)
                .margin(MARGIN)
                .constraints([
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                ])
                .split(rect.size());
            let info_layout = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(top_layout[1]);
            let main_layout = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(match player_state {
                        PlayerState::Dialogue |
                        PlayerState::Looting => 12,
                        PlayerState::None => 0,
                    }),
                ])
                .split(top_layout[0]);
            let state_layout = Layout::default()
                .direction(match player_state {
                    PlayerState::Dialogue => tui::layout::Direction::Vertical,
                    PlayerState::Looting => tui::layout::Direction::Horizontal,
                    PlayerState::None => tui::layout::Direction::Vertical,
                })
                .constraints([
                    Constraint::Percentage(match player_state {
                        PlayerState::Dialogue => 50,
                        PlayerState::Looting => 50,
                        PlayerState::None => 0,
                    }),
                    Constraint::Percentage(match player_state {
                        PlayerState::Dialogue => 50,
                        PlayerState::Looting => 50,
                        PlayerState::None => 0,
                    }),
                ])
                .split(main_layout[2]);

            // Main View
            match &data.active_menu {
                Menu::World => {
                    match player_state {
                        PlayerState::Dialogue => {
                            let dialogue = app.world.resource::<Dialogue>();
                            let p = Paragraph::new(dialogue.text.to_string())
                                .block(Block::default().borders(Borders::TOP | Borders::RIGHT | Borders::LEFT).title("Dialogue"));
                            rect.render_widget(p, state_layout[0]);

                            let mut options_items = Vec::<ListItem>::with_capacity(dialogue.options.len());
                            for (text, _) in &dialogue.options {
                                options_items.push(ListItem::new(Text::raw(text)));
                            }
                            let options = List::new(options_items)
                                .block(Block::default().borders(Borders::ALL).title("Options"))
                                .highlight_symbol(">");
                            let mut active = ListState::default();
                            active.select(Some(data.active_option.index));
                            rect.render_stateful_widget(options, state_layout[1], &mut active);
                        },
                        PlayerState::Looting => {
                            let loot_menu = app.world.resource::<LootMenu>();
                            if let Some(loot_entity) = loot_menu.inventory {
                                if let Some(inventory) = app.world.entity(loot_entity).get::<Inventory>() {
                                    // Lootable Inventory
                                    let loot_title = if matches!(data.active_option.focus, Focus::Other) {
                                        "Loot [Focused]"
                                    } else {
                                        "Loot"
                                    };
                                    let list = render_inventory(inventory, loot_title);
                                    let mut active = ListState::default();
                                    active.select(if matches!(data.active_option.focus, Focus::Other) {
                                        Some(data.active_option.index)
                                    } else {
                                        None
                                    });
                                    rect.render_stateful_widget(list, state_layout[0], &mut active);

                                    // Player Inventory
                                    let mut query = app.world.query_filtered::<&Inventory, With<PlayerTag>>();
                                    let inventory = query.single(&app.world);
                                    let inventory_title = if matches!(data.active_option.focus, Focus::Ours) {
                                        "Your Inventory [Focused]"
                                    } else {
                                        "Your Inventory"
                                    };
                                    let list = render_inventory(inventory, inventory_title);
                                    let mut active = ListState::default();
                                    active.select(if matches!(data.active_option.focus, Focus::Ours) {
                                        Some(data.active_option.index)
                                    } else {
                                        None
                                    });
                                    rect.render_stateful_widget(list, state_layout[1], &mut active);
                                }
                            }
                        },
                        PlayerState::None => {},
                    }
                    let mut player_position_query = app.world.query_filtered::<&Position, With<PlayerTag>>();
                    if let Ok(position) = player_position_query.get_single(&app.world) {
                        let vision_position = position.clone();
                        let center_position = app.world.resource::<CameraData>().position;
                        let canvas = MapCanvas {
                            world: &mut app.world,
                            center_position,
                            vision_position,
                        };
                        rect.render_widget(canvas, main_layout[1]);
                    }
                },
                Menu::Inventory => {
                    let mut query = app.world.query_filtered::<&Inventory, With<PlayerTag>>();
                    if let Ok(player_inventory) = query.get_single(&app.world) {
                        let item_list = render_inventory(player_inventory, "Inventory");
                        let mut camera_data = app.world.resource::<CameraData>().inventory_selection.clone();
                        rect.render_stateful_widget(item_list, main_layout[1], &mut camera_data);
                        app.world.resource_mut::<CameraData>().inventory_selection = camera_data;
                    }
                },
                Menu::Settings => {
                    let p = Paragraph::new("<ESC> to quit")
                        .block(Block::default().borders(Borders::ALL).title("Settings"));
                    rect.render_widget(p, main_layout[1]);
                },
            }

            // Tabs
            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(tui::style::Color::LightBlue)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(tui::style::Color::White)),
                    ])
                })
                .collect();
            let tabs = Tabs::new(menu)
                .select(usize::from(&data.active_menu))
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(tui::style::Color::White))
                .highlight_style(Style::default().fg(tui::style::Color::Yellow))
                .divider("|");

            rect.render_widget(tabs, main_layout[0]);

            // Info
            let mut info_text = String::new();
            let mut action_history_query = app.world.query_filtered::<&ActionHistory, With<PlayerTag>>();
            for action_history in action_history_query.iter(&app.world) {
                info_text.push_str(&action_history.to_string());
            }
            let info = Paragraph::new(info_text)
                .block(Block::default().borders(Borders::ALL).title("Info"));
            rect.render_widget(info, info_layout[0]);

            if let Ok((health, equipment)) = app.world.query_filtered::<(&Health, &Equipment), With<PlayerTag>>().get_single(&app.world) {
                let equipped_text = if let Some(equipped) = &equipment.equipped {
                    format!("Equipped: {}", equipped.get_name())
                } else {
                    "Equipped: None".to_string()
                };
                let health_text = format!("Health: {}/{}", health.value, health.max);
                let stats_info = Paragraph::new(format!("{health_text}\n{equipped_text}"))
                    .block(Block::default().borders(Borders::ALL).title("Stats"));
                rect.render_widget(stats_info, info_layout[1]);
            }
        })?;

        match rx.recv()? {
            Event::Input(input) => {
                if let event::Event::Key(key) = input {
                    let mut switch_menu = |menu: &mut Menu| {
                        let mut set_menu = |m: Menu, s: Scene| {
                            if *menu == m { return; }
                            *menu = m;
                            let mut scene = app.world.resource_mut::<State<Scene>>();
                            if let Err(_) = scene.overwrite_set(s) {
                                scene.clear_schedule();
                            }
                        };
                        match key.code {
                            event::KeyCode::Char('w') | event::KeyCode::Char('W') => set_menu(Menu::World, Scene::Map),
                            event::KeyCode::Char('i') | event::KeyCode::Char('I') => set_menu(Menu::Inventory, Scene::Inventory),
                            event::KeyCode::Char('s') | event::KeyCode::Char('S') => set_menu(Menu::Settings, Scene::Settings),
                            _ => {},
                        }
                    };
                    match data.active_menu {
                        Menu::World => {
                            let set_player_input_movement = |app: &mut App, movement_input: MovementInput| {
                                let player_state = app.world.resource::<PlayerState>();
                                if !matches!(player_state, PlayerState::None) { return; }
                                let mut player_input = app.world.resource_mut::<PlayerInput>();
                                (*player_input).input_movement = movement_input;
                                app.update();
                                player_input = app.world.resource_mut::<PlayerInput>();
                                (*player_input).input_movement = MovementInput::Idle;
                            };
                            match key.code {
                                event::KeyCode::Up => {
                                    let player_state = app.world.resource::<PlayerState>();
                                    match player_state {
                                        PlayerState::Dialogue |
                                        PlayerState::Looting => data.active_option.decrement(),
                                        PlayerState::None => set_player_input_movement(app, MovementInput::North),
                                    }
                                },
                                event::KeyCode::Right => set_player_input_movement(app, MovementInput::East),
                                event::KeyCode::Down => {
                                    let player_state = app.world.resource::<PlayerState>();
                                    match player_state {
                                        PlayerState::Dialogue => {
                                            let dialogue = app.world.resource::<Dialogue>();
                                            data.active_option.increment(dialogue.options.len());
                                        },
                                        PlayerState::Looting => {
                                            match data.active_option.focus {
                                                Focus::Ours => {
                                                    let mut query = app.world.query_filtered::<&Inventory, With<PlayerTag>>();
                                                    let player_inventory = query.single(&app.world);
                                                    data.active_option.increment(player_inventory.items().len());
                                                },
                                                Focus::Other => {
                                                    let loot_menu = app.world.resource::<LootMenu>();
                                                    if let Some(loot) = loot_menu.inventory {
                                                        if let Some(loot) = app.world.entity(loot).get::<Inventory>() {
                                                            data.active_option.increment(loot.items().len());
                                                        }
                                                    }
                                                },
                                            }
                                        },
                                        PlayerState::None => set_player_input_movement(app, MovementInput::South),
                                    }
                                },
                                event::KeyCode::Left => set_player_input_movement(app, MovementInput::West),
                                event::KeyCode::Enter => {
                                    let player_state = app.world.resource::<PlayerState>();
                                    match *player_state {
                                        PlayerState::Dialogue => {
                                            let copied_player_state = player_state.clone();
                                            let mut dialogue = app.world.resource_mut::<Dialogue>();
                                            if let Some(index) = data.active_option.get_index(dialogue.options.len()) {
                                                let new_player_state = dialogue.select(copied_player_state, index);
                                                data.active_option.check(dialogue.options.len());
                                                let mut player_state = app.world.resource_mut::<PlayerState>();
                                                *player_state = new_player_state;
                                            }
                                        },
                                        PlayerState::Looting => {
                                            let loot_menu = app.world.resource::<LootMenu>();
                                            if let Some(loot_inventory_entity) = loot_menu.inventory {
                                                let count = match data.active_option.focus {
                                                    Focus::Ours => {
                                                        let (player_inventory_entity, inventory) = app.world.query_filtered::<(Entity, &Inventory), With<PlayerTag>>().single(&app.world);
                                                        Some((player_inventory_entity, loot_inventory_entity, inventory.items().len()))
                                                    },
                                                    Focus::Other => {
                                                        if let Some(loot_inventory) = app.world.entity(loot_inventory_entity).get::<Inventory>() {
                                                            let count = loot_inventory.items().len();
                                                            let player_inventory_entity = app.world.query_filtered::<Entity, With<PlayerTag>>().single(&app.world);
                                                            Some((loot_inventory_entity, player_inventory_entity, count))
                                                        } else {
                                                            None
                                                        }
                                                    },
                                                };
                                                if let Some(count) = count {
                                                    if let Some(index) = data.active_option.get_index(count.2) {
                                                        transfer_item(app, (count.0, index), count.1);
                                                        if let Some(from_inventory) = app.world.entity(count.0).get::<Inventory>() {
                                                            data.active_option.check(from_inventory.items().len());
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        PlayerState::None => {},
                                    }
                                },
                                event::KeyCode::Char(' ') => {
                                    let mut query = app.world.query_filtered::<&Equipment, With<PlayerTag>>();
                                    if let Ok(equipment) = query.get_single(&app.world) {
                                        if equipment.equipped.is_some() {
                                            let mut action_input = app.world.resource_mut::<ActionInput>();
                                            *action_input = ActionInput::UseEquippedItem;
                                            app.update();
                                        }
                                    }
                                },
                                event::KeyCode::Tab => {
                                    // Select other inventory in loot menu!
                                    data.active_option.focus = match data.active_option.focus {
                                        Focus::Ours => Focus::Other,
                                        Focus::Other => Focus::Ours,
                                    };
                                    data.active_option.check_from_focus(&mut app.world);
                                },
                                event::KeyCode::Esc => {
                                    let mut player_state = app.world.resource_mut::<PlayerState>();
                                    match *player_state {
                                        PlayerState::Dialogue => {},
                                        PlayerState::Looting => {
                                            *player_state = PlayerState::None;
                                            let mut loot_menu = app.world.resource_mut::<LootMenu>();
                                            loot_menu.close();
                                            data.active_option.index = 0;
                                        },
                                        PlayerState::None => {},
                                    }
                                },
                                _ => switch_menu(&mut data.active_menu),
                            }
                        },
                        Menu::Inventory => {
                            match key.code {
                                event::KeyCode::Up => {
                                    let mut camera_data = app.world.resource_mut::<CameraData>();
                                    if let Some(current_value) = camera_data.inventory_selection.selected() {
                                        if current_value != 0 {
                                            camera_data.inventory_selection.select(Some(current_value - 1));
                                        }
                                    } else {
                                        camera_data.inventory_selection.select(Some(0));
                                    }
                                },
                                event::KeyCode::Down => {
                                    if let Ok(inventory) = app.world.query_filtered::<&Inventory, With<PlayerTag>>().get_single(&app.world) {
                                        let item_count = inventory.items().len();
                                        let mut camera_data = app.world.resource_mut::<CameraData>();
                                        if let Some(current_value) = camera_data.inventory_selection.selected() {
                                            let new_value = current_value + 1;
                                            if new_value < item_count {
                                                camera_data.inventory_selection.select(Some(new_value));
                                            }
                                        } else {
                                            camera_data.inventory_selection.select(Some(0));
                                        }
                                    }
                                },
                                event::KeyCode::Enter => {
                                    let camera_data = app.world.resource::<CameraData>();
                                    if let Some(current_value) = camera_data.inventory_selection.selected() {
                                        let mut action_input = app.world.resource_mut::<ActionInput>();
                                        *action_input = ActionInput::SelectFromInventory(current_value);
                                        app.update();
                                        if let Ok(inventory) = app.world.query_filtered::<&Inventory, With<PlayerTag>>().get_single(&app.world) {
                                            let count = inventory.items().len();
                                            if current_value >= count {
                                                let mut camera_data = app.world.resource_mut::<CameraData>();
                                                camera_data.inventory_selection.select(Some(count - 1));
                                                let player_state = app.world.resource::<PlayerState>();
                                                match player_state {
                                                    PlayerState::Looting => if matches!(data.active_option.focus, Focus::Ours) {
                                                        data.active_option.check(count);
                                                    },
                                                    PlayerState::Dialogue |
                                                    PlayerState::None => {},
                                                }
                                            }
                                        }
                                    }
                                },
                                _ => switch_menu(&mut data.active_menu),
                            }
                        },
                        Menu::Settings => {
                            match key.code {
                                event::KeyCode::Esc => {
                                    // Quit Game
                                    //disable_raw_mode()?;
                                    //terminal.show_cursor()?;
                                    execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen).ok();
                                    break;
                                },
                                _ => switch_menu(&mut data.active_menu),
                            }
                        },
                    }
                }
            },
            Event::Tick => {},
        }
    }

    Ok(())
}

use std::{time::{Duration, Instant}, thread::{self, current}, sync::mpsc::Receiver};

use bevy::prelude::{App, ResMut, Query, With, CoreStage, State};
use common::{physics::*, character::{PlayerInput, MovementInput, PlayerTag, Health, ActionHistory}, dialogue::Dialogue, inventory::{Inventory, Item, Equipment}, ActionInput, Scene};
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

use crate::canvas::MapCanvas;

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

enum Menu {
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
    selection: ListState,
}
fn update_camera_system(mut camera: ResMut<CameraData>, query: Query<&Position, With<PlayerTag>>) {
    for position in query.iter() {
        camera.position = position.clone();
    }
}

pub fn runner(mut app: App) {
    setup_game(&mut app).expect("setup_game");
}
fn setup_game(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    app
        .insert_resource(CameraData::default())
        .add_startup_system(update_camera_system)
        .add_system_to_stage(CoreStage::PostUpdate, update_camera_system);

    // Input
    let rx = setup_terminal()?;
    execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen).ok();
    let backend = CrosstermBackend::new(std::io::stdout());

    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["World", "Inventory", "Settings"];
    let mut active_menu_item = Menu::World;

    // Render
    loop {
        terminal.draw(|rect| {
            const MARGIN: u16 = 2;
            // Layout
            let dialogue = app.world.resource::<Dialogue>();
            let top_layout = Layout::default()
                .direction(tui::layout::Direction::Horizontal)
                .margin(MARGIN)
                .constraints([
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                ])
                .split(rect.size());
            let main_layout = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(if dialogue.in_conversation { 6 } else { 0 }),
                ])
                .split(top_layout[0]);

            // Main View
            match &active_menu_item {
                Menu::World => {
                    let dialogue = app.world.resource::<Dialogue>();
                    if dialogue.in_conversation {
                        let p = Paragraph::new(dialogue.text.to_string())
                            .block(Block::default().borders(Borders::ALL).title("Dialogue"));
                        rect.render_widget(p, main_layout[2]);
                    }
                    let map = app.world.resource::<Map>();
                    let camera = app.world.resource::<CameraData>();
                    let canvas = MapCanvas { map, center_position: camera.position };
                    rect.render_widget(canvas, main_layout[1]);
                },
                Menu::Inventory => {
                    let mut query = app.world.query_filtered::<&Inventory, With<PlayerTag>>();
                    if let Ok(player_inventory) = query.get_single(&app.world) {
                        let mut items = Vec::<ListItem>::with_capacity(player_inventory.items.len());
                        for item in &player_inventory.items {
                            items.push(ListItem::new(Text::raw(item.get_name())));
                        }
                        let item_list = List::new(items)
                            .block(Block::default().borders(Borders::ALL).title("Inventory"))
                            .highlight_symbol(">");
                        let mut camera_data = app.world.resource_mut::<CameraData>();
                        rect.render_stateful_widget(item_list, main_layout[1], &mut camera_data.selection);
                    }
                },
                Menu::Settings => {
                    let p = Paragraph::new("<ESC> to quit")
                        .block(Block::default().borders(Borders::ALL).title("Settings"));
                    rect.render_widget(p, main_layout[1])
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
                .select(usize::from(&active_menu_item))
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
            rect.render_widget(info, top_layout[1]);
        })?;

        match rx.recv()? {
            Event::Input(input) => {
                if let event::Event::Key(key) = input {
                    let mut switch_menu = |menu: &mut Menu| {
                        let mut set_menu = |m: Menu, s: Scene| {
                            *menu = m;
                            let mut scene = app.world.resource_mut::<State<Scene>>();
                            scene.set(s).ok();
                        };
                        match key.code {
                            event::KeyCode::Char('w') | event::KeyCode::Char('W') => set_menu(Menu::World, Scene::Map),
                            event::KeyCode::Char('i') | event::KeyCode::Char('I') => set_menu(Menu::Inventory, Scene::Inventory),
                            event::KeyCode::Char('s') | event::KeyCode::Char('S') => set_menu(Menu::Settings, Scene::Settings),
                            _ => {},
                        }
                    };
                    match active_menu_item {
                        Menu::World => {
                            let set_player_input_movement = |app: &mut App, movement_input: MovementInput| {
                                let dialogue = app.world.resource::<Dialogue>();
                                if dialogue.in_conversation { return; }
                                {
                                    let mut player_input = app.world.resource_mut::<PlayerInput>();
                                    (*player_input).input_movement = movement_input;
                                }
                                app.update();
                                let mut player_input = app.world.resource_mut::<PlayerInput>();
                                (*player_input).input_movement = MovementInput::Idle;
                            };
                            match key.code {
                                event::KeyCode::Up => {
                                    let mut dialogue = app.world.resource_mut::<Dialogue>();
                                    if dialogue.in_conversation {
                                        dialogue.decrement();
                                    } else {
                                        set_player_input_movement(app, MovementInput::North);
                                    }
                                },
                                event::KeyCode::Right => set_player_input_movement(app, MovementInput::East),
                                event::KeyCode::Down => {
                                    let mut dialogue = app.world.resource_mut::<Dialogue>();
                                    if dialogue.in_conversation {
                                        dialogue.increment();
                                    }
                                    set_player_input_movement(app, MovementInput::South);
                                },
                                event::KeyCode::Left => set_player_input_movement(app, MovementInput::West),
                                event::KeyCode::Enter => {
                                    let mut dialogue = app.world.resource_mut::<Dialogue>();
                                    if dialogue.in_conversation {
                                        dialogue.select();
                                    }
                                },
                                _ => switch_menu(&mut active_menu_item),
                            }
                        },
                        Menu::Inventory => {
                            match key.code {
                                event::KeyCode::Up => {
                                    let mut camera_data = app.world.resource_mut::<CameraData>();
                                    if let Some(current_value) = camera_data.selection.selected() {
                                        if current_value != 0 {
                                            camera_data.selection.select(Some(current_value - 1));
                                        }
                                    } else {
                                        camera_data.selection.select(Some(0));
                                    }
                                },
                                event::KeyCode::Down => {
                                    if let Ok(inventory) = app.world.query_filtered::<&Inventory, With<PlayerTag>>().get_single(&app.world) {
                                        let item_count = inventory.items.len();
                                        let mut camera_data = app.world.resource_mut::<CameraData>();
                                        if let Some(current_value) = camera_data.selection.selected() {
                                            let new_value = current_value + 1;
                                            if new_value < item_count {
                                                camera_data.selection.select(Some(new_value));
                                            }
                                        } else {
                                            camera_data.selection.select(Some(0));
                                        }
                                    }
                                },
                                event::KeyCode::Enter => {
                                    let camera_data = app.world.resource::<CameraData>();
                                    if let Some(current_value) = camera_data.selection.selected() {
                                        let mut action_input = app.world.resource_mut::<ActionInput>();
                                        *action_input = ActionInput::SelectFromInventory(current_value);
                                    }
                                },
                                _ => switch_menu(&mut active_menu_item),
                            }
                            app.update();
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
                                _ => switch_menu(&mut active_menu_item),
                            }
                            app.update();
                        },
                    }
                }
            },
            Event::Tick => {},
        }
    }

    Ok(())
}

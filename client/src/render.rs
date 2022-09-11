use std::{time::{Duration, Instant}, thread, sync::mpsc::Receiver};

use bevy::prelude::{App, State};
use common::{physics::*, character::{PlayerInput, MovementInput}, dialogue::Dialogue, inventory::Inventory};
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode}, event, execute,
};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Layout, Constraint},
    widgets::{Paragraph, Block, Borders, Tabs, List, ListItem},
    style::{Style, Modifier},
    text::{Spans, Span, Text}
};

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
            Menu::World { .. } => 0,
            Menu::Inventory => 1,
            Menu::Settings => 2,
        }
    }
}

pub fn runner<const X: usize, const Y: usize>(mut app: App) {
    setup_game::<X, Y>(&mut app).expect("setup_game");
}
fn setup_game<const X: usize, const Y: usize>(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
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
            // Layout
            let dialogue = app.world.resource::<Dialogue>();
            let top_layout = Layout::default()
                .direction(tui::layout::Direction::Horizontal)
                .margin(2)
                .constraints([
                    Constraint::Min(6),
                    Constraint::Length(if dialogue.in_conversation { 30 } else { 0 }),
                ])
                .split(rect.size());
            let main_layout = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ])
                .split(top_layout[0]);

            // Main View
            match &active_menu_item {
                Menu::World => {
                    let dialogue = app.world.resource::<Dialogue>();
                    if dialogue.in_conversation {
                        let dialogue_layout = Layout::default()
                            .direction(tui::layout::Direction::Horizontal)
                            .constraints([Constraint::Min(30)])
                            .split(top_layout[1]);
                        let p = Paragraph::new(dialogue.text.to_string())
                            .block(Block::default().borders(Borders::ALL).title("Dialogue"));
                        rect.render_widget(p, dialogue_layout[0]);
                    }
                    let map = app.world.resource::<Map<X, Y>>();
                    let mut text = String::with_capacity((X * Y) + Y);
                    for y in 0..Y {
                        for x in 0..X {
                            if let Some(tile) = map.get(x, Y - 1 - y) {
                                let character = match tile {
                                    Tile::Ground { occupier: occupier_option, .. } => {
                                        if let Some(occupier) = occupier_option {
                                            occupier.sprite.character
                                        } else {
                                            '%'
                                        }
                                    },
                                    Tile::Wall => '#',
                                };
                                text.push(character);
                            }
                        }
                        text.push('\n');
                    }
                    let p = Paragraph::new(text)
                        .block(Block::default().borders(Borders::ALL).title("World"));

                    rect.render_widget(p, main_layout[1]);
                },
                Menu::Inventory => {
                    let player_inventory = app.world.resource::<Inventory>();
                    let mut items = Vec::<ListItem>::with_capacity(player_inventory.items.len());
                    for item in &player_inventory.items {
                        items.push(ListItem::new(Text::raw(item.get_name())));
                    }
                    let p = List::new(vec![]);
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
                .divider(Span::raw("|"));

            rect.render_widget(tabs, main_layout[0]);
        })?;

        match rx.recv()? {
            Event::Input(input) => {
                if let event::Event::Key(key) = input {
                    let switch_menu = |menu: &mut Menu| {
                        match key.code {
                            event::KeyCode::Char('w') | event::KeyCode::Char('W') => *menu = Menu::World,
                            event::KeyCode::Char('i') | event::KeyCode::Char('I') => *menu = Menu::Inventory,
                            event::KeyCode::Char('s') | event::KeyCode::Char('S') => *menu = Menu::Settings,
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
                                _ => {
                                    let dialogue = app.world.resource::<Dialogue>();
                                    if !dialogue.in_conversation {
                                        switch_menu(&mut active_menu_item);
                                    }
                                },
                            }
                        },
                        Menu::Inventory => {
                            match key.code {
                                _ => switch_menu(&mut active_menu_item),
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
                                _ => switch_menu(&mut active_menu_item),
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

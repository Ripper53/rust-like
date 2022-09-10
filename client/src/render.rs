use std::{time::{Duration, Instant}, thread, sync::mpsc::Receiver};

use bevy::prelude::App;
use common::{physics::*, character::{PlayerInput, MovementInput}, dialogue::Dialogue};
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode}, event,
};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Layout, Constraint},
    widgets::{Paragraph, Block, Borders, Tabs},
    style::{Style, Modifier},
    text::{Spans, Span}
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
    World {
        dialogue: Option<Dialogue>,
    },
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

fn render_home<'a>() -> Block<'a> { Block::default() }

fn render_inventory<'a>() -> Block<'a> { Block::default() }

pub fn runner<const X: usize, const Y: usize>(mut app: App) {
    setup_game::<X, Y>(&mut app).expect("setup_game");
}
fn setup_game<const X: usize, const Y: usize>(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Input
    let rx = setup_terminal()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["World", "Inventory", "Settings"];
    let mut active_menu_item = Menu::World { dialogue: None };

    // Render
    loop {
        terminal.draw(|rect| {
            // Layout
            let size = rect.size();
            let chunks = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

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

            // Main View
            match active_menu_item {
                Menu::World { .. } => {
                    let map = app.world.resource::<Map<X, Y>>();
                    let mut text = String::with_capacity((X * Y) + Y);
                    for y in 0..Y {
                        for x in 0..X {
                            if let Some(tile) = map.get(x, Y - 1 - y) {
                                let character = match tile {
                                    Tile::Ground { sprite: sprite_option, .. } => {
                                        if let Some(sprite) = sprite_option {
                                            sprite.character
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

                    rect.render_widget(p, chunks[1]);
                },
                Menu::Inventory => rect.render_widget(render_inventory(), chunks[1]),
                Menu::Settings => {},
            }

            let tabs = Tabs::new(menu)
            .select(usize::from(&active_menu_item))
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(tui::style::Color::White))
            .highlight_style(Style::default().fg(tui::style::Color::Yellow))
            .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
        })?;

        match rx.recv()? {
            Event::Input(input) => {
                if let event::Event::Key(key) = input {
                    let switch_menu = |menu: &mut Menu| {
                        match key.code {
                            event::KeyCode::Char('w') | event::KeyCode::Char('W') => *menu = Menu::World { dialogue: None },
                            event::KeyCode::Char('i') | event::KeyCode::Char('I') => *menu = Menu::Inventory,
                            event::KeyCode::Char('s') | event::KeyCode::Char('S') => *menu = Menu::Settings,
                            _ => {},
                        }
                    };
                    match active_menu_item {
                        Menu::World { dialogue: ref mut dialogue_option } => {
                            let mut set_player_input_movement = |movement_input: MovementInput| {
                                if dialogue_option.is_some() { return; }
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
                                    if let Some(dialogue) = dialogue_option {
                                        if dialogue.active != 0 {
                                            dialogue.active -= 1;
                                        }
                                    } else {
                                        set_player_input_movement(MovementInput::North);
                                    }
                                },
                                event::KeyCode::Right => set_player_input_movement(MovementInput::East),
                                event::KeyCode::Down => {
                                    if let Some(dialogue) = dialogue_option {
                                        if dialogue.active != dialogue.options.len() {
                                            dialogue.active += 1;
                                        }
                                    } else {
                                        set_player_input_movement(MovementInput::South);
                                    }
                                },
                                event::KeyCode::Left => set_player_input_movement(MovementInput::West),
                                event::KeyCode::Enter => {
                                    if let Some(dialogue) = dialogue_option {
                                        let option = &dialogue.options[dialogue.active].0;
                                        println!("{:?}", option);
                                    }
                                },
                                _ => {
                                    if dialogue_option.is_none() {
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
                                    disable_raw_mode()?;
                                    terminal.show_cursor()?;
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

use std::{time::{Duration, Instant}, thread, sync::mpsc::Receiver};

use bevy::prelude::App;
use common::{physics::*, character::{PlayerInput, MovementInput}};
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode}, event,
};
use tui::{backend::CrosstermBackend, Terminal, layout::{Layout, Constraint}, widgets::{Paragraph, Block, Borders, Tabs}, style::{Style, Modifier}, text::{Spans, Span}};

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

#[derive(Clone, Copy)]
enum Menu {
    World,
    Inventory,
    Settings,
}
impl From<Menu> for usize {
    fn from(input: Menu) -> Self {
        match input {
            Menu::World => 0,
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
    let mut active_menu_item = Menu::World;

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

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(tui::style::Color::White))
                .highlight_style(Style::default().fg(tui::style::Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);

            // Main View
            match active_menu_item {
                Menu::World => {
                    let map = app.world.resource::<Map<X, Y>>();
                    let mut x = 0;
                    let mut y = 0;
                    let mut text = String::with_capacity((X * Y) + Y);
                    while y < Y {
                        while x < X {
                            if let Some(tile) = map.get(x, Y - 1 - y) {
                                match tile {
                                    Tile::Ground(sprite_option) => {
                                        if let Some(sprite) = sprite_option {
                                            text.push(sprite.character);
                                        } else {
                                            text.push('%');
                                        }
                                    },
                                    Tile::Wall => text.push('#'),
                                }
                            }
                            x += 1;
                        }
                        text.push('\n');
                        x = 0;
                        y += 1;
                    }
                    let p = Paragraph::new(text)
                        .block(Block::default().borders(Borders::ALL).title("World"));

                    rect.render_widget(p, chunks[1]);
                },
                Menu::Inventory => rect.render_widget(render_inventory(), chunks[1]),
                Menu::Settings => {},
            }
        })?;

        match rx.recv()? {
            Event::Input(input) => {
                if let event::Event::Key(key) = input {
                    let switch_menu = |a: &mut Menu| {
                        match key.code {
                            event::KeyCode::Char('w') => *a = Menu::World,
                            event::KeyCode::Char('i') => *a = Menu::Inventory,
                            event::KeyCode::Char('s') => *a = Menu::Settings,
                            _ => {},
                        }
                    };
                    match active_menu_item {
                        Menu::World => {
                            let mut set_player_input_movement = |movement_input: MovementInput| {
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
                                    set_player_input_movement(MovementInput::North);
                                },
                                event::KeyCode::Right => {
                                    set_player_input_movement(MovementInput::East);
                                },
                                event::KeyCode::Down => {
                                    set_player_input_movement(MovementInput::South);
                                },
                                event::KeyCode::Left => {
                                    set_player_input_movement(MovementInput::West);
                                },
                                _ => switch_menu(&mut active_menu_item),
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

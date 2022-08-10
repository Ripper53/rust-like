use std::{io::{stdout, Stdout}, time::{Duration, Instant}, thread, sync::mpsc::Receiver};

use bevy::prelude::{Query, Res, App};
use common::{physics::*, character::*};
use crossterm::{
    execute,
    style::{Color, Print, SetForegroundColor, SetBackgroundColor, ResetColor}, terminal::{enable_raw_mode, disable_raw_mode}, event,
};
use tui::{backend::CrosstermBackend, Terminal, layout::{Layout, Constraint}, widgets::{Paragraph, Block, Borders, Tabs, canvas::{Canvas, MapResolution, Context}}, style::{Style, Modifier}, text::{Spans, Span}};

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
    Home,
    World,
    Inventory,
}
impl From<Menu> for usize {
    fn from(input: Menu) -> Self {
        match input {
            Menu::Home => 0,
            Menu::World => 1,
            Menu::Inventory => 2,
        }
    }
}

fn render_home<'a>() -> Block<'a> { Block::default() }

fn render_inventory<'a>() -> Block<'a> { Block::default() }

pub fn setup_game<const X: usize, const Y: usize>(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Input
    let rx = setup_terminal()?;
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Home", "World", "Inventory"];
    let mut active_menu_item = Menu::Home;

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
                                .fg(tui::style::Color::Yellow)
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
                    app.run();

                let canvas = Canvas::default()
                    .block(Block::default().borders(Borders::ALL).title("World"))
                    .paint(|ctx| {
                        ctx.draw(&tui::widgets::canvas::Map {
                            color: tui::style::Color::White,
                            resolution: MapResolution::High,
                        });
                        if let Some(map) = app.world.get_resource::<Map<X, Y>>() {
                            let mut x: usize = 0;
                            let mut y: usize = 0;
                            while x < X {
                                while y < Y {
                                    if let Some(tile) = map.get(x, y) {
                                        match tile {
                                            Tile::Ground => ctx.print(x as f64, y as f64, Spans::from(vec![Span::styled("%", Style::default())])),
                                            Tile::Wall => ctx.print(x as f64, y as f64, Spans::from(vec![Span::styled("#", Style::default())])),
                                        }
                                    }
                                    y += 1;
                                }
                                x += 1;
                                y = 0;
                            }
                        }
                    })
                    .x_bounds([0.0, X as f64])
                    .y_bounds([0.0, Y as f64]);

                    rect.render_widget(canvas, chunks[1]);
                },
                Menu::Inventory => rect.render_widget(render_inventory(), chunks[1]),

                Menu::Home => rect.render_widget(render_home(), chunks[1]),
            }
        })?;

        match rx.recv()? {
            Event::Input(input) => {
                if let event::Event::Key(key) = input {
                    match key.code {
                        event::KeyCode::Esc => {
                            // Quit Game
                            disable_raw_mode().expect("Disabled.");
                            terminal.show_cursor().expect("Closed terminal.");
                            break;
                        },
                        event::KeyCode::Char('w') => active_menu_item = Menu::World,
                        event::KeyCode::Char('i') => active_menu_item = Menu::Inventory,
                        _ => {},
                    }
                }
            },
            Event::Tick => {},
        }
    }

    Ok(())
}

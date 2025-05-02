mod totp;

use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distr::Alphanumeric, Rng};
//use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};
//use thiserror::Error;
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
//use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};



const DB_PATH: &str = "./data/db.json";

enum Event<I> {
    Input(I),
    Tick,
}



#[derive(Serialize, Deserialize, Clone, Debug)]
struct Entry {
    id: usize,
    name: String,
    secret: String,
    created_at: String,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Entries,
    Add,
    Settings,
}

enum InputMode {
    Normal,
    Editing,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Entries => 1,
            MenuItem::Add => 2,
            MenuItem::Settings => 3,
        }
    }
}




struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut app = App::default();


    let secrets = ["ZZB53LB7PKWWT2M7LHGA2GEAIQAK26GS", "ZZB57MFSPKWWT2M7TKKA2GEAIQAK26GS"];
    for secret in secrets {
        println!("{:?}", totp::totp(secret, 30, 6))
    }


    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(500);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Home","Entries","Add", "Settings", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut entry_list_state = ListState::default();
    entry_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
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

            let copyright = Paragraph::new("totp-CLI by aviag3n/Altharion")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Copyright")
                        .border_type(BorderType::Plain),
                );

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            match active_menu_item {
                MenuItem::Home => {
                    rect.render_widget(render_home(), chunks[1]);
                    app.input_mode = InputMode::Normal;
                
                },
                MenuItem::Entries => {
                    app.input_mode = InputMode::Normal;

                    let entry_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_entries(&entry_list_state);
                    rect.render_stateful_widget(left, entry_chunks[0], &mut entry_list_state);
                    rect.render_widget(right, entry_chunks[1]);
                    
                },
                MenuItem::Add => {
                    app.input_mode = InputMode::Editing;
                    rect.render_widget(render_add_entry(&app), chunks[1]);

                    
                },
                MenuItem::Settings => {
                    
                    app.input_mode = InputMode::Normal;

                }
            }
            rect.render_widget(copyright, chunks[2]);





        })?;

        match rx.recv()? {
            Event::Input(event) => match app.input_mode {
                
                    InputMode::Normal => match event.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                        KeyCode::Char('e') => active_menu_item = MenuItem::Entries,
                        KeyCode::Char('a') => active_menu_item = MenuItem::Add,
                        KeyCode::Char('s') => active_menu_item = MenuItem::Settings,
                        
                        
                        KeyCode::Char('r') => {
                            add_random_entry_to_db().expect("shoulde be able to add new random entry");
                        },

                        KeyCode::Down => {
                            if let Some(selected) = entry_list_state.selected() {
                                let amount_entries = read_db().expect("can fetch entry list").len();
                                if selected >= amount_entries - 1 {
                                    entry_list_state.select(Some(0));
                                } else {
                                    entry_list_state.select(Some(selected + 1));
                                }
                            }
                        }
                        KeyCode::Up => {
                            if let Some(selected) = entry_list_state.selected() {
                                let amount_entries = read_db().expect("can fetch entry list").len();
                                if selected > 0 {
                                    entry_list_state.select(Some(selected - 1));
                                } else {
                                    entry_list_state.select(Some(amount_entries - 1));
                                }
                            }
                        }
                        
                        _ => {}
                    },
                    
                    InputMode::Editing => match event.code {
                        KeyCode::Enter => {
                            app.messages.push(app.input.drain(..).collect());
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                
            },
            Event::Tick => {}
        }
    }







    terminal.clear()?;
    disable_raw_mode()?;
    terminal.show_cursor()?;


    Ok(())
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            r"_        _                _____ _      _____ ",
            Style::default().fg(Color::Magenta), //Red
        )]),
        Spans::from(vec![Span::styled(
            r"| |      | |              / ____| |    |_   _|",
            Style::default().fg(Color::Magenta), //red
        )]),
        Spans::from(vec![Span::styled(
            r"| |_ ___ | |_ _ __ ______| |    | |      | |  ",
            Style::default().fg(Color::Magenta), //yellow
        )]),
        Spans::from(vec![Span::styled(
            r"| __/ _ \| __| '_ \______| |    | |      | |  ",
            Style::default().fg(Color::Magenta), //green
        )]),
        Spans::from(vec![Span::styled(
            r"| || (_) | |_| |_) |     | |____| |____ _| |_ ",
            Style::default().fg(Color::Magenta),//blue
        )]),
        Spans::from(vec![Span::styled(
            r" \__\___/ \__| .__/       \_____|______|_____|",
            Style::default().fg(Color::Magenta),//magenta
        )]),
        Spans::from(vec![Span::styled(
            r"             | |                              ",
            Style::default().fg(Color::Magenta),//lightmagenta
        )]),
        Spans::from(vec![Span::styled(
            r"             |_|                              ",
            Style::default().fg(Color::Magenta),//white
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'e' to view entries, 'a' to add a new entry, 'r' to add a random entry and 's' access settings")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}

fn render_entries<'a>(entry_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let entries = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("entries")
        .border_type(BorderType::Plain);

    let mut entry_list = read_db().expect("can fetch entry list");
    let mut items: Vec<_> = entry_list
        .iter()
        .map(|entry| {
            ListItem::new(Spans::from(vec![Span::styled(
                entry.name.clone(),
                Style::default(),
            )]))
        })
        .collect();


    if items.len() == 0 {

        add_new_entry_to_db("Placeholder", "ZZB53LB7PKWWT2M7LHGA2GEAIQAK26GS").expect("Should be able to add Placheolder Entry");

        entry_list = read_db().expect("can fetch entry list");
        items  = entry_list
        .iter()
        .map(|entry| {
            ListItem::new(Spans::from(vec![Span::styled(
                entry.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    }


    let selected_entry = entry_list
        .get(
            entry_list_state
                .selected()
                .expect("there is always a selected entry"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(entries).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let entry_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_entry.id.to_string())),
        Cell::from(Span::raw(selected_entry.name)),
        Cell::from(Span::raw(totp::totp(&selected_entry.secret, 30, 6))),
        Cell::from(Span::raw(selected_entry.created_at.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Code",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ]);

    (list, entry_detail)
}

fn read_db() -> Result<Vec<Entry>, std::io::Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Entry> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}



fn render_add_entry<'a>(app:&'a App) -> Paragraph<'a>{
    let input = Paragraph::new(app.input.as_ref())
    .style(match app.input_mode {
        InputMode::Normal => Style::default(),
        InputMode::Editing => Style::default().fg(Color::Yellow),
    })
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Add Entry")
            .border_type(BorderType::Plain),
    );

    input
}


fn add_new_entry_to_db(name:&str, secret:&str) -> Result<Vec<Entry>, std::io::Error> {
    let mut rng = rand::rng();
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Entry> = serde_json::from_str(&db_content)?;

    let entry =Entry {
        id: rng.random_range(0..9999999),
        name: name.to_string(),
        secret: secret.to_string(),
        created_at: Utc::now().format("%d/%m/%Y").to_string(),
    };

    //println!("Generated random entry: {:?}", random_entry);

    // Add the random entry to the list and write it back to the file
    parsed.push(entry);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}



fn add_random_entry_to_db() -> Result<Vec<Entry>, std::io::Error> {
    let mut rng = rand::rng();
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Entry> = serde_json::from_str(&db_content)?;

    let random_entry =Entry {
        id: rng.random_range(0..9999999),
        name: rng.clone().sample_iter(Alphanumeric).take(10).map(char::from).collect(),
        secret: rng.sample_iter(Alphanumeric).take(32).map(char::from).collect(),
        created_at: Utc::now().format("%d/%m/%Y").to_string(),
    };

    //println!("Generated random entry: {:?}", random_entry);

    // Add the random entry to the list and write it back to the file
    parsed.push(random_entry);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}


/*
fn remove_entry_at_index(entry_list_state: &mut ListState) -> Result<(), std::io::Error> {
    if let Some(selected) = entry_list_state.selected() {
        let db_content = fs::read_to_string(DB_PATH)?;
        let mut parsed: Vec<Entry> = serde_json::from_str(&db_content)?;
        parsed.remove(selected);
        fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
        let amount_entries = read_db().expect("can fetch entry list").len();
        if selected > 0 {
            entry_list_state.select(Some(selected - 1));
        } else {
            entry_list_state.select(Some(0));
        }
    }
    Ok(())
}
    */
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use crate::db::{Db, HistoryEntry};

struct App {
    input: String,
    results: Vec<HistoryEntry>,
    list_state: ListState,
    db: Db,
}

impl App {
    fn new(db: Db, initial_query: Option<String>) -> Self {
        let mut app = Self {
            input: initial_query.unwrap_or_default(),
            results: Vec::new(),
            list_state: ListState::default(),
            db,
        };
        app.refresh_results();
        app
    }

    fn refresh_results(&mut self) {
        if let Ok(results) = self.db.search_entries(&self.input) {
            self.results = results;
            self.list_state.select(Some(0));
        }
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.results.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.results.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

pub fn run(db: Db, initial_query: Option<String>) -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(db, initial_query);

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(Some(selected_cmd)) = res {
        // Print the selected command to stdout so the shell can eval it
        // We need to print it AFTER restoring the terminal
        println!("{}", selected_cmd);
    } else if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<Option<String>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    if let Some(i) = app.list_state.selected() {
                        if let Some(entry) = app.results.get(i) {
                            return Ok(Some(entry.command.clone()));
                        }
                    }
                }
                KeyCode::Esc => return Ok(None),
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                KeyCode::Char(c) => {
                    app.input.push(c);
                    app.refresh_results();
                }
                KeyCode::Backspace => {
                    app.input.pop();
                    app.refresh_results();
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.size());

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(input, chunks[0]);

    let items: Vec<ListItem> = app
        .results
        .iter()
        .map(|i| {
            let timestamp = i.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            let exit_mark = if i.exit_code.unwrap_or(0) == 0 { "✅" } else { "❌" };
            let content = vec![
                Line::from(vec![
                    Span::styled(format!("{} ", exit_mark), Style::default()),
                    Span::styled(format!("[{}] ", timestamp), Style::default().fg(Color::DarkGray)),
                    Span::raw(&i.command),
                ])
            ];
            ListItem::new(content)
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("History"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(items, chunks[1], &mut app.list_state);
}

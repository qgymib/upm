mod search_tab;

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};

#[derive(Debug)]
pub struct App {
    activate_tab: Tab,
    search_tab: search_tab::SearchTab,
}

#[derive(Debug, Default)]
enum Tab {
    #[default]
    Search,
    InstallationSummary,
}

impl App {
    pub fn new() -> Self {
        Self {
            activate_tab: Tab::Search,
            search_tab: search_tab::SearchTab::new(),
        }
    }
}

pub fn app() -> anyhow::Result<()> {
    std::io::stdout().execute(ratatui::crossterm::terminal::EnterAlternateScreen)?;
    ratatui::crossterm::terminal::enable_raw_mode()?;

    let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
    let mut terminal = ratatui::Terminal::new(backend)?;
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    std::io::stdout().execute(ratatui::crossterm::terminal::LeaveAlternateScreen)?;
    ratatui::crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn handle_events() -> anyhow::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Draw the UI of the application.
///
/// # Arguments
/// * `frame` - The frame to draw the UI on.
fn ui(frame: &mut ratatui::Frame) {
    frame.render_widget(
        Paragraph::new("Hello World!").block(Block::bordered().title("Greeting")),
        frame.size(),
    );
}

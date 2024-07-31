use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        terminal::{
            disable_raw_mode, enable_raw_mode,
            EnterAlternateScreen, LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    Terminal,
};
use std::io::{self, stdout};

pub fn init() -> io::Result<Terminal<impl Backend>> {
    _ = stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> io::Result<()> {
    _ = stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

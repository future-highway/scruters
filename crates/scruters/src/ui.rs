use crate::state::State;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw(_state: &State, frame: &mut Frame<'_>) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(frame.size());

    let text = vec![
        Line::raw("Hello, world!"),
        Line::raw("Press 'q' to quit."),
    ];

    let text = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    frame.render_widget(text, chunks[0]);
}

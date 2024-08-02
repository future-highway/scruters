use crate::state::State;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::border,
    widgets::{Block, Widget as _},
    Frame,
};
use tracing::error;
use tui_logger::{TuiLoggerWidget, TuiWidgetState};

pub fn draw(state: &mut State, frame: &mut Frame<'_>) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(frame.size());

    let Some(tui_logger_area) = areas.first().copied()
    else {
        error!("No area for TuiLoggerWidget");
        unreachable!("No area for TuiLoggerWidget");
    };

    let Some(action_bar_area) = areas.get(1).copied()
    else {
        error!("No area for action bar");
        unreachable!("No area for action bar");
    };

    draw_tui_logger_widget(
        state,
        tui_logger_area,
        frame.buffer_mut(),
    );

    super::draw_action_bar(
        &[("<esc>", "back")],
        action_bar_area,
        frame.buffer_mut(),
    );
}

fn draw_tui_logger_widget(
    _state: &mut State,
    area: Rect,
    buf: &mut Buffer,
) {
    let state = TuiWidgetState::new();

    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .title("")
        .title(" Logs ");

    // TODO: Colorize logs
    TuiLoggerWidget::default()
        .block(block)
        .style(Style::default().fg(Color::White))
        .output_separator(' ')
        .state(&state)
        .render(area, buf);
}

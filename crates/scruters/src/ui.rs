use crate::state::{Screen, State};
use ratatui::Frame;

mod logs;
mod testing;

pub fn draw(state: &mut State, frame: &mut Frame<'_>) {
    let Some(screen) = state.current_screen.as_ref() else {
        return;
    };

    match screen {
        Screen::Logs => logs::draw(state, frame),
        Screen::Testing => testing::draw(state, frame),
    }
}

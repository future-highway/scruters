use crate::state::{Screen, State};
use itertools::Itertools;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget as _},
    Frame,
};

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

fn draw_action_bar(
    actions: &[(&'static str, &'static str)],
    area: Rect,
    buf: &mut Buffer,
) {
    let block =
        Block::default().padding(Padding::symmetric(1, 0));

    #[allow(unstable_name_collisions)]
    let actions = actions
        .iter()
        .map(|(key, name)| {
            let key =
                Span::styled(*key, Style::new().bold());
            let name = Span::raw(*name);

            vec![key, Span::raw(" "), name]
        })
        .intersperse(vec![Span::raw(" | ")])
        .flatten()
        .collect::<Vec<_>>();

    let actions = Line::from(actions);
    let actions = Paragraph::new(actions).block(block);

    actions.render(area, buf);
}

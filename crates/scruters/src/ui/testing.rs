use crate::state::{testing::ActiveComponent, State};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    widgets::{Block, List, ListItem, StatefulWidget},
    Frame,
};

pub fn draw(state: &mut State, frame: &mut Frame<'_>) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(frame.size());

    let Some(groups_area) = areas.first().copied() else {
        tracing::error!("No area for groups");
        unreachable!("No area for groups");
    };

    let Some(action_bar_area) = areas.get(1).copied()
    else {
        tracing::error!("No area for action bar");
        unreachable!("No area for action bar");
    };

    draw_groups_widget(
        state,
        groups_area,
        frame.buffer_mut(),
    );

    super::draw_action_bar(
        &[("<r>", "run")],
        action_bar_area,
        frame.buffer_mut(),
    );
}

fn draw_groups_widget(
    state: &mut State,
    area: Rect,
    buf: &mut Buffer,
) {
    let testing_state = &mut state.testing_state;
    let is_active = matches!(
        testing_state.active_component,
        ActiveComponent::Groups
    );

    let border_style = if is_active {
        Style::new().green()
    } else {
        Style::default()
    };

    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(border_style)
        .title("")
        .title(" Groups ");

    let mut list = List::new(Vec::<ListItem<'_>>::new())
        .block(block)
        .highlight_symbol("> ");

    if is_active {
        list = list.style(Style::new().blue());
    }

    list.render(
        area,
        buf,
        &mut testing_state.groups_component_state,
    );
}

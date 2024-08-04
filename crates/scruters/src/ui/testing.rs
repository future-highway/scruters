use crate::state::{
    testing::{
        groups::AnyGroup, ActiveComponent, TestingState,
    },
    State,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, ListItem, StatefulWidget},
    Frame,
};

#[allow(clippy::cognitive_complexity)]
pub fn draw(state: &mut State, frame: &mut Frame<'_>) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .split(frame.size());

    let Some(main_area) = areas.first().copied() else {
        tracing::error!("No main area");
        unreachable!("No main area");
    };

    let Some(action_bar_area) = areas.get(1).copied()
    else {
        tracing::error!("No area for action bar");
        unreachable!("No area for action bar");
    };

    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Fill(1),
        ])
        .split(main_area);

    let Some(groups_area) = areas.first().copied() else {
        tracing::error!("No groups area");
        unreachable!("No groups area");
    };

    let Some(testing_area) = areas.get(1).copied() else {
        tracing::error!("No testing area");
        unreachable!("No testing area");
    };

    draw_groups_widget(
        state,
        groups_area,
        frame.buffer_mut(),
    );

    draw_testing_widget(
        state,
        testing_area,
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

    let list_items = testing_state
        .groups
        .iter()
        .map(|group| group.name())
        .collect::<Vec<_>>();

    let list_items = list_items
        .iter()
        .map(AsRef::as_ref)
        .map(|group| ListItem::new(Line::raw(group)))
        .collect::<Vec<_>>();

    let mut list = List::new(list_items)
        .block(block)
        .highlight_symbol("  ");

    if is_active {
        list = list.highlight_style(Style::new().on_blue());
    }

    list.render(
        area,
        buf,
        &mut testing_state.groups_component_state,
    );
}

fn draw_testing_widget(
    state: &mut State,
    area: Rect,
    buf: &mut Buffer,
) {
    let TestingState {
        active_component,
        groups_component_state,
        tests_component_state,
        groups,
        ..
    } = &mut state.testing_state;

    let is_active =
        matches!(active_component, ActiveComponent::Tests);

    let border_style = if is_active {
        Style::new().green()
    } else {
        Style::default()
    };

    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(border_style)
        .title("")
        .title(" Tests ");

    let selected_group = groups_component_state
        .selected()
        .and_then(|index| groups.get(index));

    let list_items = selected_group
        .map(|group| {
            group
                .tests()
                .iter()
                .map(|test_name| {
                    ListItem::new(Line::raw(
                        test_name.as_str(),
                    ))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut list = List::new(list_items)
        .block(block)
        .highlight_symbol("  ");

    if is_active {
        list = list.highlight_style(Style::new().on_blue());
    }

    list.render(area, buf, tests_component_state);
}

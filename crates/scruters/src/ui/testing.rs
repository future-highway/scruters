use crate::state::{
    testing::{
        groups::AnyGroup, tests::AnyTest, ActiveComponent,
        OutputSource, TestingState,
    },
    State,
};
use alloc::borrow::Cow;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{
        Block, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget, Wrap,
    },
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

    let Some(areas) = areas.get(1).copied() else {
        tracing::error!("No right side area");
        unreachable!("No right side area");
    };

    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Fill(1),
        ])
        .split(areas);

    let Some(testing_area) = areas.first().copied() else {
        tracing::error!("No testing area");
        unreachable!("No testing area");
    };

    let Some(output_area) = areas.get(1).copied() else {
        tracing::error!("No output area");
        unreachable!("No output area");
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

    draw_output_widget(
        state,
        output_area,
        frame.buffer_mut(),
    );

    let actions = match state.testing_state.active_component
    {
        ActiveComponent::Groups => {
            vec![
                ("<enter>", "select"),
                ("<r>", "run group"),
            ]
        }
        ActiveComponent::Tests => {
            vec![("<esc>", "back"), ("<r>", "run test")]
        }
        ActiveComponent::Output(_) => {
            vec![("<esc>", "back")]
        }
    };

    super::draw_action_bar(
        &actions,
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
        .title(" [1] Groups ");

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
        .highlight_symbol("> ");

    if is_active {
        list = list.highlight_style(Style::new().on_blue());
    }

    StatefulWidget::render(
        list,
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
        tests_output,
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
        .title(" [2] Tests ");

    let selected_group = groups_component_state
        .selected()
        .and_then(|index| groups.get(index));

    let selected_group_tests = selected_group
        .map(AnyGroup::tests)
        .unwrap_or_default();

    let selected_group_test_names = selected_group_tests
        .iter()
        .map(|test| test.name())
        .collect::<Vec<_>>();

    let selected_group_test_names =
        selected_group_test_names
            .iter()
            .map(Cow::as_ref)
            .collect::<Vec<_>>();

    let list_items = selected_group_tests
        .iter()
        .zip(selected_group_test_names)
        .map(|(test, name)| {
            let status_mark =
                tests_output.get(test).map_or_else(
                    || Span::raw("?").yellow(),
                    |(passed, _)| {
                        if *passed {
                            Span::raw("✓").green()
                        } else {
                            Span::raw("✗").red()
                        }
                    },
                );

            let test_name = Span::raw(name.as_str());

            ListItem::new(Line::from(vec![
                status_mark,
                Span::raw(" "),
                test_name,
            ]))
        })
        .collect::<Vec<_>>();

    let mut list = List::new(list_items)
        .block(block)
        .highlight_symbol("> ");

    if is_active {
        list = list.highlight_style(Style::new().on_blue());
    }

    StatefulWidget::render(
        list,
        area,
        buf,
        tests_component_state,
    );
}

fn draw_output_widget(
    state: &mut State,
    area: Rect,
    buf: &mut Buffer,
) {
    let TestingState {
        active_component,
        groups_component_state,
        tests_component_state,
        groups,
        tests_output,
        ..
    } = &mut state.testing_state;

    let is_active = matches!(
        active_component,
        ActiveComponent::Output(_)
    );

    let border_style = if is_active {
        Style::new().green()
    } else {
        Style::default()
    };

    let title = match active_component {
        ActiveComponent::Groups
        | ActiveComponent::Output(OutputSource::Groups) => {
            " [3] Group Output "
        }
        ActiveComponent::Tests
        | ActiveComponent::Output(OutputSource::Tests) => {
            " [3] Test Output "
        }
    };

    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(border_style)
        .title("")
        .title(title);

    let seleced_group = groups_component_state
        .selected()
        .and_then(|index| groups.get(index));

    let lines = match active_component {
        ActiveComponent::Groups
        | ActiveComponent::Output(OutputSource::Groups) => {
            seleced_group.and_then(|group| group.output())
        }
        ActiveComponent::Tests
        | ActiveComponent::Output(OutputSource::Tests) => {
            let selected_test =
                seleced_group.and_then(|group| {
                    let tests = group.tests();
                    tests_component_state
                        .selected()
                        .and_then(|index| tests.get(index))
                });

            selected_test
                .and_then(|test| tests_output.get(test))
                .map(|(_, lines)| &**lines)
        }
    }
    .unwrap_or_default();

    let lines =
        lines.iter().map(Line::raw).collect::<Vec<_>>();

    let lines_count = lines.len();

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(block);

    let scrollbar =
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .track_symbol(Some("│"))
            .thumb_symbol("|")
            .thumb_style(Style::new().bold())
            .end_symbol(Some("▼"));

    let mut scollbar_state =
        ScrollbarState::new(lines_count).position(0);

    Widget::render(paragraph, area, buf);
    scrollbar.render(area, buf, &mut scollbar_state);
}

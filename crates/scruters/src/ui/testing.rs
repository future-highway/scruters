use crate::state::{
    testing::{
        groups::AnyGroup, ActiveComponent, OutputSource,
    },
    State,
};
use alloc::borrow::Cow;
use ansi_to_tui::IntoText;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Margin, Rect},
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

const HIGHLIGHT_SYMBOL: &str = "> ";
const STATUS_MARK_PASSED: &str = "✓";
const STATUS_MARK_FAILED: &str = "✗";
const STATUS_MARK_NOT_RUN: &str = "?";

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
            vec![("<esc>", "back"), ("<r>", "re-run")]
        }
    };

    super::draw_action_bar(
        &actions,
        action_bar_area,
        frame.buffer_mut(),
    );
}

#[allow(clippy::too_many_lines)]
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
        .title(" <1> Groups ");

    let groups = testing_state
        .groups
        .iter()
        .map(|group| {
            let status =
                testing_state.get_group_status(group);

            (group.name(), status)
        })
        .collect::<Vec<_>>();

    let list_items = groups
        .iter()
        .map(|(name, status)| {
            let status = match status {
                Some(true) => {
                    Span::raw(STATUS_MARK_PASSED).green()
                }
                Some(false) => {
                    Span::raw(STATUS_MARK_FAILED).red()
                }
                None => {
                    Span::raw(STATUS_MARK_NOT_RUN).yellow()
                }
            };

            let name = Span::raw(name.as_str());

            ListItem::new(Line::from(vec![
                status,
                Span::raw(" "),
                name,
            ]))
        })
        .collect::<Vec<_>>();

    let mut list = List::new(list_items)
        .block(block)
        .highlight_symbol(HIGHLIGHT_SYMBOL);

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
    let is_active = matches!(
        state.testing_state.active_component,
        ActiveComponent::Tests
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
        .title(" <2> Tests ");

    let selected_group = state
        .testing_state
        .groups_component_state
        .selected()
        .and_then(|index| {
            state.testing_state.groups.get(index)
        });

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
            let status_mark = state
                .testing_state
                .get_test_status(test)
                .map_or_else(
                    || {
                        Span::raw(STATUS_MARK_NOT_RUN)
                            .yellow()
                    },
                    |passed| {
                        if passed {
                            Span::raw(STATUS_MARK_PASSED)
                                .green()
                        } else {
                            Span::raw(STATUS_MARK_FAILED)
                                .red()
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
        .highlight_symbol(HIGHLIGHT_SYMBOL);

    if is_active {
        list = list.highlight_style(Style::new().on_blue());
    }

    StatefulWidget::render(
        list,
        area,
        buf,
        &mut state.testing_state.tests_component_state,
    );
}

fn draw_output_widget(
    state: &mut State,
    area: Rect,
    buf: &mut Buffer,
) {
    let testing_state = &mut state.testing_state;

    let is_active = matches!(
        testing_state.active_component,
        ActiveComponent::Output(_)
    );

    let border_style = if is_active {
        Style::new().green()
    } else {
        Style::default()
    };

    let title = match testing_state.active_component {
        ActiveComponent::Groups
        | ActiveComponent::Output(OutputSource::Groups) => {
            " <3> Group Output "
        }
        ActiveComponent::Tests
        | ActiveComponent::Output(OutputSource::Tests) => {
            " <3> Test Output "
        }
    };

    let block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(border_style)
        .title("")
        .title(title);

    let seleced_group = testing_state
        .groups_component_state
        .selected()
        .and_then(|index| testing_state.groups.get(index));

    let lines = match testing_state.active_component {
        ActiveComponent::Groups
        | ActiveComponent::Output(OutputSource::Groups) => {
            seleced_group.and_then(|group| {
                testing_state.get_group_output(group)
            })
        }
        ActiveComponent::Tests
        | ActiveComponent::Output(OutputSource::Tests) => {
            let selected_test =
                seleced_group.and_then(|group| {
                    let tests = group.tests();
                    testing_state
                        .tests_component_state
                        .selected()
                        .and_then(|index| tests.get(index))
                });

            selected_test.and_then(|test| {
                testing_state.get_test_output(test)
            })
        }
    }
    .unwrap_or_default();

    let lines = lines
        .iter()
        .flat_map(IntoText::into_text)
        .flat_map(|text| text.lines)
        .collect::<Vec<_>>();

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

    let mut scollbar_state = ScrollbarState::new(
        lines_count,
    )
    .position(state.testing_state.output_scroll_position);

    Widget::render(paragraph, area, buf);
    scrollbar.render(
        area.inner(Margin { vertical: 1, horizontal: 0 }),
        buf,
        &mut scollbar_state,
    );
}

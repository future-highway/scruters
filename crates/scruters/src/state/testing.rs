pub(crate) use self::{
    active_component::ActiveComponent, test_name::TestName,
};
use super::helpers::default_list_state;
use crate::message::{Message, TestingMessage};
use color_eyre::{eyre::Context as _, Result};
use core::time::Duration;
use crossterm::event::{KeyCode, KeyEvent};
use groups::{run_group, Group, GroupName};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::UnboundedSender, task::JoinHandle,
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::debug;

mod active_component;
pub(crate) mod groups;
mod test_name;

#[allow(clippy::partial_pub_fields)]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TestingState {
    #[serde(skip, default)]
    pub active_component: ActiveComponent,
    #[serde(skip, default = "default_list_state")]
    pub groups_component_state: ListState,
    #[serde(skip, default = "default_list_state")]
    pub tests_component_state: ListState,
    #[serde(skip, default)]
    pub groups: Vec<Group>,
    #[serde(skip, default)]
    task: Option<(JoinHandle<()>, CancellationToken)>,
}

impl Default for TestingState {
    fn default() -> Self {
        Self {
            active_component: ActiveComponent::Groups,
            groups_component_state: default_list_state(),
            tests_component_state: default_list_state(),
            groups: Vec::new(),
            task: None,
        }
    }
}

impl TestingState {
    pub(super) fn add_auto_generated_groups(
        &mut self,
        message_tx: UnboundedSender<Message>,
    ) {
        let mut groups =
            Group::all_auto_generated_groups(message_tx);

        groups.append(&mut self.groups);
        self.groups = groups;
    }

    pub(super) fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> Option<Message> {
        #[allow(clippy::wildcard_enum_match_arm)]
        match key_event.code {
            KeyCode::Char('r')
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::RunGroup,
                ))
            }
            KeyCode::Down
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::NextGroup,
                ))
            }
            KeyCode::End
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::LastGroup,
                ))
            }
            KeyCode::Home
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::FirstGroup,
                ))
            }
            KeyCode::Up
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::PreviousGroup,
                ))
            }
            _ => {
                debug!(?key_event, "Unhandled key event");
                None
            }
        }
    }

    pub(super) async fn handle_message(
        &mut self,
        message: TestingMessage,
        message_tx: UnboundedSender<Message>,
    ) -> Result<Option<Message>> {
        match message {
            TestingMessage::FirstGroup => {
                self.groups_component_state.select_first();
            }
            TestingMessage::GroupRunOutput(
                group_name,
                line,
            ) => {
                let Some(group) =
                    self.get_group_mut(&group_name)
                else {
                    debug!(
                        ?group_name,
                        "Received auto generated group tests for group that does not exist"
                    );

                    return Ok(None);
                };

                group.push_output(line);
            }
            TestingMessage::LastGroup => {
                self.groups_component_state.select_last();
            }
            TestingMessage::NextGroup => {
                self.groups_component_state.select_next();
            }
            TestingMessage::PreviousGroup => {
                self.groups_component_state
                    .select_previous();
            }
            TestingMessage::ReplaceGroupTests(
                group_name,
                tests,
            ) => {
                let Some(group) =
                    self.get_group_mut(&group_name)
                else {
                    debug!(
                        ?group_name,
                        "Received auto generated group tests for group that does not exist"
                    );

                    return Ok(None);
                };

                group.replace_tests(tests);
            }
            TestingMessage::RunGroup => {
                self.cancel_running_task().await;

                let Some(group) = self.selected_group()
                else {
                    debug!(
                        "Attempted to run group, but no group was selected"
                    );

                    return Ok(None);
                };

                self.task = Some(
                    run_group(group, message_tx)
                        .wrap_err("Failed to run group")?,
                );
            }
        }

        Ok(None)
    }

    async fn cancel_running_task(&mut self) {
        if let Some((task, cancellation_token)) =
            self.task.take()
        {
            if task.is_finished() {
                return;
            }

            cancellation_token.cancel();
            let mut counter = 0_i32;
            while !task.is_finished() {
                sleep(Duration::from_millis(1)).await;

                counter = counter.saturating_add(1);
                if counter > 50_i32 {
                    task.abort();
                }
                if counter > 100_i32 {
                    break;
                }
            }
        }
    }

    fn selected_group(&self) -> Option<&Group> {
        self.groups_component_state
            .selected()
            .and_then(|i| self.groups.get(i))
    }

    fn get_group_mut(
        &mut self,
        group_name: &GroupName,
    ) -> Option<&mut Group> {
        self.groups
            .iter_mut()
            .find(|group| &group.name == group_name)
    }
}

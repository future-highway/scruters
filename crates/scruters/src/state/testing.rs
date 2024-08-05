pub(crate) use self::{
    active_component::ActiveComponent, test_name::TestName,
};
use super::helpers::default_list_state;
use crate::{
    message::{Message, TestingMessage},
    state::testing::groups::auto_generated_groups::auto_generated_group_metadata,
};
use alloc::borrow::Cow;
use cargo_metadata::Metadata;
use color_eyre::{eyre::Context as _, Result};
use core::{fmt::Debug, time::Duration};
use crossterm::event::{KeyCode, KeyEvent};
use futures::{stream, StreamExt as _};
use groups::{
    auto_generated_groups::AutoGeneratedGroup, run_group,
    AnyGroup, Group, GroupName, Groups,
};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{mpsc::UnboundedSender, watch},
    task::JoinHandle,
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

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
    #[serde(default)]
    pub groups: Groups,
    #[serde(skip, default)]
    task: Option<(JoinHandle<()>, CancellationToken)>,
}

impl Default for TestingState {
    fn default() -> Self {
        Self {
            active_component: ActiveComponent::Groups,
            groups_component_state: default_list_state(),
            tests_component_state: default_list_state(),
            groups: Groups::default(),
            task: None,
        }
    }
}

impl TestingState {
    pub(super) fn init(
        &mut self,
        metadata_rx: watch::Receiver<Metadata>,
        message_tx: UnboundedSender<Message>,
    ) {
        // We want the initial groups even without the tests
        auto_generated_group_metadata::all_from_metadata(
            &metadata_rx.borrow(),
        )
        .into_iter()
        .map(AutoGeneratedGroup::empty_from_metadata)
        .for_each(|group| {
            self.groups
                .push_front(Group::AutoGenerated(group));
        });

        // We need the groups to be sorted for the binary
        // search to work
        self.groups.make_contiguous().sort_unstable();

        // We want to keep the auto generated groups up to
        // date, so we spawn a task to watch for changes
        drop(tokio::spawn(watch_auto_generated_groups(
            metadata_rx,
            message_tx,
        )));
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
                    TestingMessage::RunSelectedGroup,
                ))
            }
            KeyCode::Down
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::SelectNextGroup,
                ))
            }
            KeyCode::Esc
                if self.active_component
                    == ActiveComponent::Tests =>
            {
                Some(Message::Testing(
                    TestingMessage::SetActiveComponent(
                        ActiveComponent::Groups,
                    ),
                ))
            }
            KeyCode::End
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::SelectLastGroup,
                ))
            }
            KeyCode::Enter
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::SetActiveComponent(
                        ActiveComponent::Tests,
                    ),
                ))
            }
            KeyCode::Home
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::SelectFirstGroup,
                ))
            }
            KeyCode::Up
                if self.active_component
                    == ActiveComponent::Groups =>
            {
                Some(Message::Testing(
                    TestingMessage::SelectPreviousGroup,
                ))
            }
            _ => None,
        }
    }

    pub(super) async fn handle_message(
        &mut self,
        message: TestingMessage,
        message_tx: UnboundedSender<Message>,
    ) -> Result<Option<Message>> {
        match message {
            TestingMessage::GroupRunOutput(
                group_name,
                line,
            ) => {
                let group_name = Cow::Owned(group_name);
                let Ok(index) =
                    self.groups.binary_search_by(|group| {
                        group.name().cmp(&group_name)
                    })
                else {
                    debug!(
                        ?group_name,
                        "GroupName not found"
                    );

                    return Ok(None);
                };

                let Some(group) =
                    self.groups.get_mut(index)
                else {
                    unreachable!(
                        "Group index not found following successful binary search"
                    );
                };

                group.push_output(line);
            }
            TestingMessage::RetainAutoGeneratedGroups(
                group_names,
            ) => {
                self.groups.retain(|group| match group {
                    Group::Custom(_) => true,
                    Group::AutoGenerated(group) => {
                        group_names.contains(&group.name())
                    }
                });
            }
            TestingMessage::RunSelectedGroup => {
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
            TestingMessage::SetActiveComponent(
                component,
            ) => {
                self.active_component = component;
            }
            TestingMessage::SelectFirstGroup => {
                self.groups_component_state.select_first();
            }
            TestingMessage::SelectLastGroup => {
                self.groups_component_state.select_last();
            }
            TestingMessage::SelectNextGroup => {
                self.groups_component_state.select_next();
            }
            TestingMessage::SelectPreviousGroup => {
                self.groups_component_state
                    .select_previous();
            }
            TestingMessage::UpsertGroup(group) => {
                match self.groups.binary_search(&group) {
                    Ok(index) => {
                        let Some(existing_group) =
                            self.groups.get_mut(index)
                        else {
                            unreachable!(
                                "Group index not found following successful binary search"
                            );
                        };

                        existing_group
                            .set_tests(group.into_tests());
                    }
                    Err(i) => {
                        self.groups.insert(i, group);
                    }
                }

                if self
                    .tests_component_state
                    .selected()
                    .is_none()
                {
                    self.tests_component_state
                        .select_first();
                }
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

    pub(crate) fn selected_group(&self) -> Option<&Group> {
        self.groups_component_state
            .selected()
            .and_then(|index| self.groups.get(index))
    }
}

async fn watch_auto_generated_groups(
    mut metadata_rx: watch::Receiver<Metadata>,
    message_tx: UnboundedSender<Message>,
) {
    loop {
        debug!(
            "Metadata changed...updating auto generated groups"
        );

        let metadata = {
            auto_generated_group_metadata::all_from_metadata(
                &metadata_rx.borrow_and_update(),
            )
        };

        let names_to_retain =
            metadata.iter().map(GroupName::from).collect();

        if let Err(error) =
            message_tx.send(Message::Testing(
                TestingMessage::RetainAutoGeneratedGroups(
                    names_to_retain,
                ),
            ))
        {
            error!(
                ?error,
                "Error sending message to retain auto generated groups"
            );

            return;
        }

        let cancellation_token = CancellationToken::new();
        let cancellation_token_clone =
            cancellation_token.clone();
        let tx = message_tx.clone();

        #[allow(clippy::integer_division_remainder_used)]
        drop(tokio::spawn(async move {
            tokio::select! {
                () = cancellation_token_clone.cancelled() => {
                    debug!("Cancelling auto generated group creation");
                }
                () = stream::iter(metadata)
                    .for_each(|metadata| async {
                        match AutoGeneratedGroup::from_metadata(
                            metadata,
                            cancellation_token_clone.clone(),
                        )
                        .await {
                            Ok(Some(group)) => {
                                if let Err(error) = tx.send(Message::Testing(
                                    TestingMessage::UpsertGroup(group.into()),
                                )) {
                                    error!(
                                        ?error,
                                        "Error sending auto generated group"
                                    );
                                }
                            }
                            Ok(None) => {}
                            Err(error) => {
                                error!(?error, "Error creating auto generated group");
                            }
                        }
                    }) => {}
            }
        }));

        match metadata_rx.changed().await {
            Ok(()) => {
                cancellation_token.cancel();
            }
            Err(_) => break,
        }
    }
}

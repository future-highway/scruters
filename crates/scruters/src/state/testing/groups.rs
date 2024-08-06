use self::{
    auto_generated_groups::{
        AutoGeneratedGroup, AutoGeneratedGroupMetadata,
    },
    custom_groups::CustomGroup,
};
pub(crate) use self::{
    group::Group, group_name::GroupName,
};
use super::tests::Test;
use crate::{
    cargo::CargoTestArgs,
    command::spawn_command,
    message::{Message, TestingMessage},
};
use alloc::{borrow::Cow, collections::VecDeque};
use color_eyre::eyre::{Context, Result};
use core::ops::{Deref, DerefMut};
use futures::stream::select;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt as _, BufReader},
    sync::mpsc::UnboundedSender,
    task::JoinHandle,
};
use tokio_stream::{wrappers::LinesStream, StreamExt};
use tokio_util::sync::CancellationToken;

pub mod auto_generated_groups;
mod custom_groups;
mod group;
mod group_name;

pub(crate) trait AnyGroup {
    fn name(&self) -> Cow<'_, GroupName>;

    fn tests(&self) -> &[Test];

    fn update_group(&mut self, group: Group);

    fn to_cargo_test_args(&self) -> CargoTestArgs<'_>;

    fn push_output(&mut self, line: String);
}

pub(super) fn run_group<Group: AnyGroup>(
    group: &Group,
    messages_tx: UnboundedSender<Message>,
) -> Result<(JoinHandle<()>, CancellationToken)> {
    let cancellation_token = CancellationToken::new();

    let command = group.to_cargo_test_args().into_command();

    let (stdout, stderr) =
        spawn_command(command, cancellation_token.clone())
            .wrap_err("Failed to spawn command")?;

    let mut reader = select(
        LinesStream::new(BufReader::new(stdout).lines()),
        LinesStream::new(BufReader::new(stderr).lines()),
    );

    let group_name = group.name().into_owned();

    let join_handle = tokio::spawn(async move {
        while let Ok(Some(line)) =
            reader.next().await.transpose().map_err(|error| {
                tracing::error!(
                    ?error,
                    "Error reading line from run group command output"
                );
            })
        {
            if let Err(error) = messages_tx
                .send(Message::Testing(
                    TestingMessage::GroupRunOutput(group_name.clone(), line)
                )) {
                tracing::error!(
                    ?error,
                    "Failed to send group run output message"
                );
            }
        }
    });

    Ok((join_handle, cancellation_token))
}

#[derive(Debug, Default)]
pub(crate) struct Groups(VecDeque<Group>);

impl Serialize for Groups {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let filtered: Vec<&Group> = self
            .0
            .iter()
            .filter(|group| group.should_serialize())
            .collect();

        filtered.serialize(serializer)
    }
}

#[allow(clippy::missing_trait_methods)]
impl<'de> Deserialize<'de> for Groups {
    fn deserialize<D>(
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let groups: VecDeque<Group> =
            VecDeque::deserialize(deserializer)?;

        Ok(Self(groups))
    }
}

impl Deref for Groups {
    type Target = VecDeque<Group>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Groups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

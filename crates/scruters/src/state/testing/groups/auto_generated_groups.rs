pub(crate) use self::auto_generated_group_metadata::AutoGeneratedGroupMetadata;
use super::{AnyGroup, GroupName};
use crate::{
    cargo::CargoTestArgs,
    command::spawn_command,
    state::testing::{
        groups::Group,
        tests::{Test, TestName},
    },
};
use alloc::{borrow::Cow, collections::VecDeque};
use auto_generated_group_metadata::TargetKind;
use color_eyre::{eyre::Context, Result};
use core::cmp::Ordering;
use futures::stream::select;
use if_chain::if_chain;
use tokio::io::{AsyncBufReadExt as _, BufReader};
use tokio_stream::{wrappers::LinesStream, StreamExt};
use tokio_util::sync::CancellationToken;

pub mod auto_generated_group_metadata;

#[derive(Debug)]
pub(crate) struct AutoGeneratedGroup {
    name: GroupName,
    metadata: AutoGeneratedGroupMetadata,
    pub(super) tests: VecDeque<Test>,
    output: Option<Vec<String>>,
}

impl AutoGeneratedGroup {
    fn new(
        name: GroupName,
        metadata: AutoGeneratedGroupMetadata,
        mut tests: VecDeque<Test>,
    ) -> Self {
        tests.make_contiguous().sort_unstable();
        Self { name, metadata, tests, output: None }
    }

    pub(crate) fn empty_from_metadata(
        metadata: AutoGeneratedGroupMetadata,
    ) -> Self {
        Self::new(
            GroupName::from(&metadata),
            metadata,
            VecDeque::new(),
        )
    }
    pub(in crate::state::testing) async fn from_metadata(
        metadata: AutoGeneratedGroupMetadata,
        cancellation_token: CancellationToken,
    ) -> Result<Option<Self>> {
        const TEST_LINE_SUFFIX: &str = ": test";
        const TEST_LINE_SUFFIX_LEN: usize =
            TEST_LINE_SUFFIX.len();

        let command = CargoTestArgs {
            args: metadata.to_args().map(Into::into),
            color: false,
            list: true,
            ..Default::default()
        }
        .into_command();

        let (stdout, stderr) = spawn_command(
            command,
            cancellation_token.clone(),
        )
        .wrap_err("Failed to run command listing tests")?;

        let mut reader = select(
            LinesStream::new(
                BufReader::new(stdout).lines(),
            ),
            LinesStream::new(
                BufReader::new(stderr).lines(),
            ),
        );

        let mut test_names = Vec::new();

        #[allow(clippy::integer_division_remainder_used)]
        loop {
            tokio::select! {
                () = cancellation_token.cancelled() => {
                    return Ok(None);
                }
                maybe_line = reader.next() => {
                    if let Some(mut line) = maybe_line.transpose().wrap_err("Failed to read line from command output")? {
                        if_chain! {
                            if line.ends_with(TEST_LINE_SUFFIX);
                            let _ = line.split_off(line.len().saturating_sub(TEST_LINE_SUFFIX_LEN));
                            if !line.is_empty() && !line.contains(' ');
                            then {
                                test_names.push(line);
                            }
                        }
                    } else {
                        let group = Self::new(
                            GroupName::from(&metadata),
                            metadata,
                            test_names.into_iter().map(TestName).map(Test::Named).collect(),
                        );

                        return Ok(Some(group));
                    }
                }
            }
        }
    }
}

impl AnyGroup for AutoGeneratedGroup {
    fn name(&self) -> Cow<'_, GroupName> {
        Cow::Borrowed(&self.name)
    }

    fn tests(&self) -> &[Test] {
        self.tests.as_slices().0
    }

    fn update_group(&mut self, group: Group) {
        debug_assert!(
            self.name.eq(group.name().as_ref()),
            "Expected group names to be equal"
        );

        #[allow(clippy::panic)]
        let Group::AutoGenerated(group) = group else {
            panic!("Expected an auto-generated group");
        };

        self.metadata = group.metadata;
        self.tests = group.tests;

        let auto_generated_tests = match &self
            .metadata
            .target_kind
        {
            TargetKind::Bin(_)
            | TargetKind::Lib(_)
            | TargetKind::Test(_) => vec![],
            TargetKind::Tests(target_names) => target_names
                .iter()
                .map(|target_name| {
                    Test::IntegrationTarget {
                        package_name: self
                            .metadata
                            .package_name
                            .clone(),
                        target_name: target_name.clone(),
                    }
                })
                .collect(),
            TargetKind::Examples(target_names) => {
                target_names
                    .iter()
                    .map(|target_name| Test::Examples {
                        package_name: self
                            .metadata
                            .package_name
                            .clone(),
                        target_name: target_name.clone(),
                    })
                    .collect()
            }
            TargetKind::Example(target_name) => {
                vec![Test::Example {
                    package_name: self
                        .metadata
                        .package_name
                        .clone(),
                    target_name: target_name.clone(),
                }]
            }
        };

        for test in auto_generated_tests {
            match self.tests.binary_search(&test) {
                Ok(_) => {}
                Err(index) => {
                    self.tests.insert(index, test);
                }
            }
        }

        _ = self.tests.make_contiguous();
    }

    fn to_cargo_test_args(&self) -> CargoTestArgs<'_> {
        let args = self.metadata.to_args().map(Into::into);
        CargoTestArgs { args, ..CargoTestArgs::default() }
    }

    fn reset_output(&mut self) {
        self.output = None;
    }

    fn push_output(&mut self, line: String) {
        if let Some(output) = &mut self.output {
            output.push(line);
        } else {
            self.output = Some(vec![line]);
        }
    }

    fn output(&self) -> Option<&[String]> {
        self.output.as_deref()
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialEq for AutoGeneratedGroup {
    fn eq(&self, other: &Self) -> bool {
        self.metadata == other.metadata
    }
}

#[allow(clippy::missing_trait_methods)]
impl Eq for AutoGeneratedGroup {}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for AutoGeneratedGroup {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::missing_trait_methods)]
impl Ord for AutoGeneratedGroup {
    fn cmp(&self, other: &Self) -> Ordering {
        self.metadata.cmp(&other.metadata)
    }
}

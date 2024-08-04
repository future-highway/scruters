use super::{AnyGroup, GroupName};
use crate::{
    cargo::CargoTestArgs, state::testing::TestName,
};
use alloc::borrow::Cow;
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CustomGroup {
    name: GroupName,
    pub(super) tests: Vec<TestName>,
    #[serde(skip, default)]
    output: Option<Vec<String>>,
}

impl AnyGroup for CustomGroup {
    fn name(&self) -> Cow<'_, GroupName> {
        Cow::Borrowed(&self.name)
    }

    fn tests(&self) -> &[TestName] {
        &self.tests
    }

    fn set_tests(&mut self, tests: Vec<TestName>) {
        self.tests = tests;
    }

    fn to_cargo_test_args(&self) -> CargoTestArgs<'_> {
        todo!()
    }

    fn push_output(&mut self, line: String) {
        if let Some(output) = &mut self.output {
            output.push(line);
        } else {
            self.output = Some(vec![line]);
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialEq for CustomGroup {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[allow(clippy::missing_trait_methods)]
impl Eq for CustomGroup {}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for CustomGroup {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::missing_trait_methods)]
impl Ord for CustomGroup {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

use super::{AnyGroup, Group, GroupKey, GroupName};
use crate::{
    cargo::CargoTestArgs, state::testing::tests::Test,
};
use alloc::{borrow::Cow, collections::VecDeque};
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CustomGroup {
    name: GroupName,
    pub(super) tests: VecDeque<Test>,
}

impl AnyGroup for CustomGroup {
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
        let Group::Custom(group) = group else {
            panic!("Expected a custom group");
        };

        self.tests = group.tests;
    }

    fn to_cargo_test_args(&self) -> CargoTestArgs<'_> {
        todo!()
    }

    fn as_group_key(&self) -> GroupKey<'_> {
        GroupKey::Custom(Cow::Borrowed(&self.name))
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
impl Hash for CustomGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

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

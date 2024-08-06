pub(crate) use self::test_name::TestName;
use super::AUTO_GENERATED_MARKER;
use alloc::borrow::Cow;
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

mod test_name;

pub(crate) trait AnyTest {
    fn name(&self) -> Cow<'_, TestName>;
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Test {
    Named(TestName),
    #[serde(skip)]
    Example {
        package_name: String,
        target_name: String,
    },
}

impl AnyTest for Test {
    fn name(&self) -> Cow<'_, TestName> {
        match self {
            Self::Named(name) => Cow::Borrowed(name),
            Self::Example { .. } => Cow::Owned(
                format!("main {AUTO_GENERATED_MARKER}")
                    .into(),
            ),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialEq for Test {
    fn eq(&self, other: &Self) -> bool {
        self.name().as_ref().eq(other.name().as_ref())
    }
}

#[allow(clippy::missing_trait_methods)]
impl Eq for Test {}

#[allow(clippy::missing_trait_methods)]
impl Ord for Test {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(&other.name())
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for Test {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use crate::state::testing::AUTO_GENERATED_MARKER;
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize,
)]
pub(crate) struct TestName(pub String);

impl TestName {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for TestName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for TestName {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::missing_trait_methods)]
impl Ord for TestName {
    fn cmp(&self, other: &Self) -> Ordering {
        // Auto-generated tests should be sorted at the top
        match (
            self.0.ends_with(AUTO_GENERATED_MARKER),
            other.0.ends_with(AUTO_GENERATED_MARKER),
        ) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => self.0.cmp(&other.0),
        }
    }
}

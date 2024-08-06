use super::AutoGeneratedGroupMetadata;
use crate::state::testing::AUTO_GENERATED_MARKER;
use alloc::borrow::Cow;
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub(crate) struct GroupName(Cow<'static, str>);

impl GroupName {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'a> From<&'a GroupName> for Cow<'a, str> {
    fn from(val: &'a GroupName) -> Self {
        Cow::Borrowed(val.as_str())
    }
}

impl From<&'static str> for GroupName {
    fn from(s: &'static str) -> Self {
        Self(s.into())
    }
}

impl From<String> for GroupName {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<&AutoGeneratedGroupMetadata> for GroupName {
    fn from(metadata: &AutoGeneratedGroupMetadata) -> Self {
        format!(
            "{}::{}::{} {}",
            metadata.package_name,
            metadata.target_kind,
            metadata.target_name,
            AUTO_GENERATED_MARKER
        )
        .into()
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for GroupName {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::missing_trait_methods)]
impl Ord for GroupName {
    fn cmp(&self, other: &Self) -> Ordering {
        // Auto-generated groups should be sorted at the top
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

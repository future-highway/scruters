use alloc::borrow::Cow;
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

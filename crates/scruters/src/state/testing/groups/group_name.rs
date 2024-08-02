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

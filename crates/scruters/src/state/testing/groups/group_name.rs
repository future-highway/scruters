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
pub(crate) struct GroupName(String);

impl From<String> for GroupName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

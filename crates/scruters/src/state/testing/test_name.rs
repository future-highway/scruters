use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CustomGroup {
    #[serde(skip, default)]
    pub(crate) output: Option<Vec<String>>,
}

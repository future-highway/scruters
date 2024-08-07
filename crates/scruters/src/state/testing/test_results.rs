use super::tests::{Test, TestResult};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TestResults(HashMap<Test, TestResult>);

impl TestResults {
    pub const fn as_inner(
        &self,
    ) -> &HashMap<Test, TestResult> {
        &self.0
    }

    pub fn as_inner_mut(
        &mut self,
    ) -> &mut HashMap<Test, TestResult> {
        &mut self.0
    }
}

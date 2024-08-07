#[derive(Debug, Default)]
pub struct TestResult {
    pub passed: Option<bool>,
    pub output: Vec<String>,
}

impl TestResult {
    pub fn reset(&mut self) {
        self.passed = None;
        self.output.clear();
    }
}

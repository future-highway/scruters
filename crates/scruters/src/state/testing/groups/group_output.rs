use crate::state::testing::tests::Test;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum GroupOutputCaptureMode {
    #[default]
    Normal,
    FailedTest(Test),
}

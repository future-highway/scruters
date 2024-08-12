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
pub(crate) enum TestKind {
    Bin {
        target_name: String,
    },
    BinTest {
        target_name: String,
        test_name: String,
    },
    Lib,
    LibTest {
        test_name: String,
    },
    IntegrationTarget {
        target_name: String,
    },
    IntegrationTest {
        target_name: String,
        test_name: String,
    },
    ExampleTarget {
        target_name: String,
    },
    ExampleMain {
        target_name: String,
    },
    ExampleTest {
        target_name: String,
        test_name: String,
    },
}

pub(crate) use self::{
    test_name::TestName, test_result::TestResult,
};
use super::AUTO_GENERATED_MARKER;
use crate::cargo::CargoTestArgs;
use alloc::borrow::Cow;
use core::cmp::Ordering;
use serde::{Deserialize, Serialize};

mod test_name;
mod test_result;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
)]
pub(crate) enum Test {
    Named(TestName),
    #[serde(skip)]
    IntegrationTarget {
        package_name: String,
        target_name: String,
    },
    #[serde(skip)]
    Examples {
        package_name: String,
        target_name: String,
    },
    #[serde(skip)]
    Example {
        package_name: String,
        target_name: String,
    },
}

impl Test {
    pub fn name(&self) -> Cow<'_, TestName> {
        match self {
            Self::Named(name) => Cow::Borrowed(name),
            Self::IntegrationTarget { package_name, target_name } => Cow::Owned(
                format!("{package_name}::{target_name} {AUTO_GENERATED_MARKER}")
                    .into()
            ),
            Self::Examples { package_name, target_name } => Cow::Owned(
                format!("{package_name}::{target_name}::main")
                    .into()
            ),
            Self::Example { .. } => Cow::Owned(
                format!("main {AUTO_GENERATED_MARKER}")
                    .into(),
            ),
        }
    }

    pub fn to_cargo_test_args(&self) -> CargoTestArgs<'_> {
        match self {
            Self::Named(test_name) => CargoTestArgs {
                args: Some(
                    vec![test_name.0.clone().into()].into(),
                ),
                test_args: Some(&["--exact"]),
                ..Default::default()
            },
            Self::IntegrationTarget {
                package_name,
                target_name,
            } => CargoTestArgs {
                args: Some(
                    vec![
                        "--package".into(),
                        package_name.clone().into(),
                        "--test".into(),
                        target_name.clone().into(),
                    ]
                    .into(),
                ),
                ..Default::default()
            },
            Self::Examples { package_name, .. } => {
                CargoTestArgs {
                    args: Some(
                        vec![
                            "--package".into(),
                            package_name.clone().into(),
                            "--examples".into(),
                        ]
                        .into(),
                    ),
                    ..Default::default()
                }
            }
            Self::Example { package_name, target_name } => {
                CargoTestArgs {
                    args: Some(
                        vec![
                            "--package".into(),
                            package_name.clone().into(),
                            "--example".into(),
                            target_name.clone().into(),
                        ]
                        .into(),
                    ),
                    ..Default::default()
                }
            }
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl Ord for Test {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(&other.name())
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for Test {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

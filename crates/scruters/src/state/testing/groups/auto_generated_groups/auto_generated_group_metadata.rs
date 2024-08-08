use alloc::borrow::Cow;
use cargo_metadata::Metadata;
use core::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    mem::discriminant,
};
use std::collections::HashSet;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub(crate) struct AutoGeneratedGroupMetadata {
    pub package_name: String,
    pub target_kind: TargetKind,
}

#[derive(Debug, Clone)]
pub(crate) enum TargetKind {
    Bin(String),
    Lib(String),
    Tests(HashSet<String>),
    Test(String),
    Examples(HashSet<String>),
    Example(String),
}

#[allow(clippy::missing_trait_methods)]
impl PartialEq for TargetKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bin(a), Self::Bin(b))
            | (Self::Lib(a), Self::Lib(b))
            | (Self::Test(a), Self::Test(b))
            | (Self::Example(a), Self::Example(b)) => {
                a.eq(b)
            }
            (Self::Tests(_), Self::Tests(_))
            | (Self::Examples(_), Self::Examples(_)) => {
                true
            }
            _ => false,
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl Eq for TargetKind {}

#[allow(clippy::missing_trait_methods)]
impl Hash for TargetKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        discriminant(self).hash(state);
    }
}

#[allow(clippy::match_same_arms)]
#[allow(clippy::missing_trait_methods)]
impl Ord for TargetKind {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Bin(a), Self::Bin(b))
            | (Self::Lib(a), Self::Lib(b))
            | (Self::Test(a), Self::Test(b))
            | (Self::Example(a), Self::Example(b)) => {
                a.cmp(b)
            }
            (Self::Tests(_), Self::Tests(_))
            | (Self::Examples(_), Self::Examples(_)) => {
                Ordering::Equal
            }
            (Self::Bin(_), _) => Ordering::Less,
            (Self::Lib(_), other) => {
                if matches!(other, Self::Bin(_)) {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (Self::Tests(_), other) => {
                if matches!(
                    other,
                    Self::Bin(_) | Self::Lib(_)
                ) {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (Self::Test(_), other) => {
                if matches!(
                    other,
                    Self::Bin(_)
                        | Self::Lib(_)
                        | Self::Tests(_)
                ) {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (Self::Examples(_), other) => {
                if matches!(
                    other,
                    Self::Bin(_)
                        | Self::Lib(_)
                        | Self::Tests(_)
                        | Self::Test(_)
                ) {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (Self::Example(_), _) => Ordering::Greater,
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for TargetKind {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for TargetKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bin(target_name) => {
                write!(f, "bin::{target_name}")
            }
            Self::Lib(_target_name) => {
                // There can only be one lib target per
                // package, so we don't need to include the
                // target name.
                write!(f, "lib")
            }
            Self::Test(target_name) => {
                write!(f, "test::{target_name}")
            }
            Self::Tests(_) => write!(f, "tests"),
            Self::Example(target_name) => {
                write!(f, "example::{target_name}")
            }
            Self::Examples(_) => write!(f, "examples"),
        }
    }
}

impl AutoGeneratedGroupMetadata {
    #[allow(clippy::unnecessary_wraps)]
    pub fn to_args(
        &self,
    ) -> Option<Vec<Cow<'static, str>>> {
        let Self { package_name, target_kind } = self;

        match target_kind {
            TargetKind::Bin(target_name) => Some(vec![
                "--package".into(),
                package_name.clone().into(),
                "--bin".into(),
                target_name.clone().into(),
            ]),
            TargetKind::Lib(_) => Some(vec![
                "--package".into(),
                package_name.clone().into(),
                "--lib".into(),
            ]),
            TargetKind::Test(target_name) => Some(vec![
                "--package".into(),
                package_name.clone().into(),
                "--test".into(),
                target_name.clone().into(),
            ]),
            TargetKind::Tests(_) => Some(vec![
                "--package".into(),
                package_name.clone().into(),
                "--test".into(),
                "*".into(),
            ]),
            TargetKind::Example(target_name) => {
                // This only runs the unit tests for the
                // example, not the main function.
                Some(vec![
                    "--package".into(),
                    package_name.clone().into(),
                    "--example".into(),
                    target_name.clone().into(),
                ])
            }
            TargetKind::Examples(_) => {
                // This only runs the unit tests for the
                // examples, not the main functions.
                Some(vec![
                    "--package".into(),
                    package_name.clone().into(),
                    "--examples".into(),
                ])
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
pub(in crate::state) fn all_from_metadata(
    metadata: &Metadata,
) -> Vec<AutoGeneratedGroupMetadata> {
    let mut auto_generated_group_metadata = metadata
        .packages
        .iter()
        .flat_map(|package| {
            package.targets.iter().flat_map(|target| {
                target.kind.iter().filter_map(|kind| {
                    // "bin", "example", "test", "bench",
                    // "lib", "custom-build"

                    tracing::trace!(
                        package = %package.name,
                        target = %target.name,
                        kind = %kind,
                        "Checking target kind",
                    );

                    let target_kind = match kind.as_str() {
                        "bin" => TargetKind::Bin,
                        "lib" => TargetKind::Lib,
                        "test" => TargetKind::Test,
                        "example" => TargetKind::Example,
                        _ => return None,
                    };

                    let target_kind =
                        target_kind(target.name.clone());

                    Some(AutoGeneratedGroupMetadata {
                        package_name: package.name.clone(),
                        target_kind,
                    })
                })
            })
        })
        .collect::<Vec<_>>();

    let mut additional_groups: Vec<
        AutoGeneratedGroupMetadata,
    > = vec![];
    for metadata in &auto_generated_group_metadata {
        if let TargetKind::Test(target_name) =
            &metadata.target_kind
        {
            let existing = additional_groups
                .iter_mut()
                .find(|group| {
                    if group.package_name
                        != metadata.package_name
                    {
                        return false;
                    }

                    let TargetKind::Tests(_) =
                        &group.target_kind
                    else {
                        return false;
                    };

                    true
                });

            if let Some(group) = existing {
                let TargetKind::Tests(target_names) =
                    &mut group.target_kind
                else {
                    unreachable!(
                        "The target kind should be `TargetKind::Tests`"
                    )
                };

                _ = target_names
                    .insert(target_name.clone());
            } else {
                additional_groups.push(
                    AutoGeneratedGroupMetadata {
                        package_name: metadata
                            .package_name
                            .clone(),
                        target_kind: TargetKind::Tests(
                            [target_name.clone()].into(),
                        ),
                    },
                );
            }
        };

        if let TargetKind::Example(target_name) =
            &metadata.target_kind
        {
            let existing = additional_groups
                .iter_mut()
                .filter(|group| {
                    group.package_name
                        == metadata.package_name
                })
                .find(|group| {
                    matches!(
                        group.target_kind,
                        TargetKind::Examples(_)
                    )
                });

            if let Some(group) = existing {
                let TargetKind::Examples(target_names) =
                    &mut group.target_kind
                else {
                    unreachable!(
                        "The target kind should be `TargetKind::Examples`"
                    )
                };

                _ = target_names
                    .insert(target_name.clone());
            } else {
                additional_groups.push(
                    AutoGeneratedGroupMetadata {
                        package_name: metadata
                            .package_name
                            .clone(),
                        target_kind: TargetKind::Examples(
                            [target_name.clone()].into(),
                        ),
                    },
                );
            }
        };
    }

    auto_generated_group_metadata
        .append(&mut additional_groups);

    auto_generated_group_metadata
}

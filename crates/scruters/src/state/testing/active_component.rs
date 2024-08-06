#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum ActiveComponent {
    #[default]
    Groups,
    Tests,
    Output(OutputSource),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum OutputSource {
    Groups,
    Tests,
}

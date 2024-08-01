#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum ActiveComponent {
    #[default]
    Groups,
    Tests,
}

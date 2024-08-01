#[derive(Debug, Default)]
pub(crate) enum Screen {
    #[default]
    Testing,
    Logs,
}

#[allow(clippy::unnecessary_wraps)]
pub(super) fn default_screen() -> Option<Screen> {
    Some(Screen::default())
}

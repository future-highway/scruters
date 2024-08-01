use crossterm::event::KeyEvent;

pub(crate) enum Message {
    Quit,
    KeyEvent(KeyEvent),
    Testing(TestingMessage),
}

impl From<KeyEvent> for Message {
    fn from(event: KeyEvent) -> Self {
        Self::KeyEvent(event)
    }
}

pub(crate) enum TestingMessage {
    RunGroup,
}

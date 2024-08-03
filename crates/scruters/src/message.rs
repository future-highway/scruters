use crate::state::{
    testing::{groups::GroupName, TestName},
    Screen,
};
use crossterm::event::KeyEvent;

pub(crate) enum Message {
    KeyEvent(KeyEvent),
    GoToScreen(Screen),
    Quit,
    Testing(TestingMessage),
}

impl From<KeyEvent> for Message {
    fn from(event: KeyEvent) -> Self {
        Self::KeyEvent(event)
    }
}

pub(crate) enum TestingMessage {
    FirstGroup,
    GroupRunOutput(GroupName, String),
    LastGroup,
    NextGroup,
    PreviousGroup,
    ReplaceGroupTests(GroupName, Vec<TestName>),
    RunGroup,
}

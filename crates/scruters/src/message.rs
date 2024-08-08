use crate::state::{
    testing::{
        groups::{Group, GroupKey, GroupName},
        tests::Test,
        ActiveComponent as TestingActiveComponent,
    },
    Screen,
};
use crossterm::event::KeyEvent;
use std::collections::HashSet;

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
    OutputFromGroupRun(GroupKey<'static>, String),
    OutputFromTestRun(Test, String),
    RetainAutoGeneratedGroups(HashSet<GroupName>),
    RunSelectedGroup,
    RunSelectedTest,
    SetActiveComponent(TestingActiveComponent),
    SelectFirstGroup,
    SelectFirstTest,
    SelectLastGroup,
    SelectLastTest,
    SelectNextGroup,
    SelectNextTest,
    SelectPreviousGroup,
    SelectPreviousTest,
    UpsertGroup(Group),
}

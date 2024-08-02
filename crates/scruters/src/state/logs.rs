use super::Screen;
use crate::message::Message;
use crossterm::event::{KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[allow(clippy::empty_structs_with_brackets)]
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct LogsState {
    #[serde(skip, default)]
    pub(super) previous_screen: Screen,
}

impl LogsState {
    pub(super) fn handle_key_event(
        &mut self,
        key_event: KeyEvent,
    ) -> Option<Message> {
        #[allow(clippy::wildcard_enum_match_arm)]
        match key_event.code {
            KeyCode::Esc
                if key_event.modifiers.is_empty() =>
            {
                Some(Message::GoToScreen(
                    self.previous_screen,
                ))
            }
            _ => {
                debug!(?key_event, "Unhandled key event");
                None
            }
        }
    }
}

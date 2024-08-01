use super::{
    testing::TestingState, LoadState, SaveState, Screen,
};
use crate::message::Message;
use color_eyre::{eyre::Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    #[serde(
        skip,
        default = "super::screen::default_screen"
    )]
    pub current_screen: Option<Screen>,
    pub testing_state: TestingState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            current_screen: Some(Screen::default()),
            testing_state: TestingState::default(),
        }
    }
}

impl State {
    pub async fn load_from_file() -> Result<Option<Self>> {
        let state = LoadState::load_from_file().await?;

        let Some(state) = state else {
            return Ok(None);
        };

        match state {
            LoadState::V0(state) => Ok(Some(state)),
        }
    }

    pub async fn save_to_file(&self) -> Result<()> {
        let state = SaveState::V0(self);
        state.save_to_file().await
    }

    pub async fn handle_message(
        &mut self,
        message: Message,
    ) -> Result<Option<Message>> {
        match message {
            Message::Quit
            | Message::KeyEvent(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                self.save_to_file()
                    .await
                    .wrap_err("Error saving state")?;

                self.current_screen = None;
            }
            Message::KeyEvent(key_event) => {
                #[allow(clippy::single_match)]
                match self.current_screen {
                    Some(Screen::Testing) => {
                        return Ok(self
                            .testing_state
                            .handle_key_event(key_event));
                    }
                    _ => {}
                }
            }
            Message::Testing(message) => {
                return self
                    .testing_state
                    .handle_message(message)
                    .await;
            }
        }

        Ok(None)
    }
}

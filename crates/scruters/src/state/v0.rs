use super::{
    logs::LogsState, testing::TestingState, LoadState,
    SaveState, Screen,
};
use crate::message::Message;
use color_eyre::{eyre::Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

#[allow(clippy::struct_field_names)]
#[allow(clippy::partial_pub_fields)]
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    #[serde(
        skip,
        default = "super::screen::default_screen"
    )]
    pub current_screen: Option<Screen>,
    #[serde(default)]
    pub testing_state: TestingState,
    #[serde(default)]
    pub logs_state: LogsState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            current_screen: Some(Screen::default()),
            testing_state: TestingState::default(),
            logs_state: LogsState::default(),
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
            Message::GoToScreen(screen) => {
                self.current_screen = Some(screen);
            }
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
            Message::KeyEvent(KeyEvent {
                code: KeyCode::Char('?'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                let previous_screen = self
                    .current_screen
                    .replace(Screen::Logs);

                self.logs_state.previous_screen =
                    previous_screen.unwrap_or_default();
            }
            Message::KeyEvent(key_event) => {
                match self.current_screen {
                    Some(Screen::Testing) => {
                        return Ok(self
                            .testing_state
                            .handle_key_event(key_event));
                    }
                    Some(Screen::Logs) => {
                        return Ok(self
                            .logs_state
                            .handle_key_event(key_event));
                    }
                    None => {}
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

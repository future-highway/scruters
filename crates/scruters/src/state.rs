//! State management for the application.
//!
//! The state is saved and loaded from the 'scruters.json'
//! file. It is designed to be version controlled.

use self::{screen::Screen, testing::TestingState};
use crate::message::Message;
use color_eyre::{eyre::Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt, ErrorKind},
};

mod screen;
mod testing;

const STATE_FILE_PATH: &str = "scruters.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    #[serde(skip, default = "screen::default_screen")]
    pub current_screen: Option<Screen>,
    pub testing_state: TestingState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            current_screen: Some(Screen::default()),
            testing_state: TestingState,
        }
    }
}

impl State {
    pub async fn load_from_file() -> Result<Option<Self>> {
        let file = OpenOptions::new()
            .read(true)
            .open(STATE_FILE_PATH)
            .await;

        let file = match file {
            Ok(file) => Some(file),
            Err(error)
                if error.kind() == ErrorKind::NotFound =>
            {
                None
            }
            Err(error)
                if error.kind()
                    == ErrorKind::PermissionDenied =>
            {
                return Err(error).wrap_err(
                    "Permission denied reading scruters.json",
                );
            }
            Err(error) => {
                return Err(error).wrap_err(
                    "Error opening scruters.json for reading",
                );
            }
        };

        let Some(mut file) = file else {
            return Ok(None);
        };

        let mut buf = Vec::new();
        _ = file
            .read_to_end(&mut buf)
            .await
            .wrap_err("Error reading scruters.json")?;

        let state = serde_json::from_slice::<Self>(&buf)
            .wrap_err(
                "Error deserializing scruters.json",
            )?;

        Ok(Some(state))
    }

    pub async fn save_to_file(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)
            .wrap_err("Error serializing state")?;

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(STATE_FILE_PATH)
            .await;

        let mut file = match file {
            Ok(file) => file,
            Err(error)
                if error.kind()
                    == ErrorKind::PermissionDenied =>
            {
                return Err(error).wrap_err(
                    "Permission denied writing scruters.json",
                );
            }
            Err(error) => {
                return Err(error).wrap_err(
                    "Error opening scruters.json for writing",
                );
            }
        };

        file.write_all(json.as_bytes())
            .await
            .wrap_err("Error writing scruters.json")?;

        Ok(())
    }

    pub async fn handle_message(
        &mut self,
        message: Message,
    ) -> Result<Option<Message>> {
        match message {
            Message::Quit
            | Message::KeyEvent(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                self.save_to_file()
                    .await
                    .wrap_err("Error saving state")?;

                self.current_screen = None;
            }
            Message::KeyEvent(_) => {}
        }

        Ok(None)
    }
}

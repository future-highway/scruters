//! State management for the application.
//!
//! The state is saved and loaded from the 'scruters.json'
//! file. It is designed to be version controlled.

use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use testing::TestingState;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt, ErrorKind},
};

mod testing;

const STATE_FILE_PATH: &str = "scruters.json";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    testing_state: TestingState,
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
}

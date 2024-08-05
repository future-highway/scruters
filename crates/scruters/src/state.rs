//! State management for the application.
//!
//! The state is saved and loaded from the 'scruters.json'
//! file. It is designed to be version controlled.

pub(crate) use self::{
    screen::Screen,
    v0::{State as StateV0, State},
};
use crate::message::Message;
use cargo_metadata::MetadataCommand;
use color_eyre::{eyre::Context as _, Result};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt as _},
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::UnboundedSender,
        watch,
    },
};
use tracing::{debug, error, trace};

mod helpers;
pub(crate) mod logs;
mod screen;
pub(crate) mod testing;
mod v0;

const STATE_FILE_PATH: &str = "scruters.json";

pub async fn initialize_state(
    workspace_change_signal: broadcast::Sender<()>,
    message_tx: UnboundedSender<Message>,
) -> Result<State> {
    let mut state = if let Some(state) =
        State::load_from_file()
            .await
            .wrap_err("Error loading state")?
    {
        debug!("Loaded state from file");
        state
    } else {
        trace!("Creating new state");

        let state = State::new();

        state
            .save_to_file()
            .await
            .wrap_err("Error saving state")?;

        debug!("Saved initial state to file");

        state
    };

    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .wrap_err("Failed to get cargo metadata")?;

    let (metadata_tx, metadata_rx) =
        watch::channel(metadata);

    let mut workspace_rx =
        workspace_change_signal.subscribe();

    drop(tokio::spawn(async move {
        loop {
            match workspace_rx.recv().await {
                Ok(()) => {}
                Err(RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }

            match MetadataCommand::new().no_deps().exec() {
                Ok(metadata) => {
                    if let Err(error) =
                        metadata_tx.send(metadata)
                    {
                        error!(
                            ?error,
                            "Failed to send metadata to watch handler",
                        );
                    }
                }
                Err(error) => {
                    error!(
                        ?error,
                        "Failed to get cargo metadata",
                    );
                }
            };
        }
    }));

    state.init(metadata_rx, message_tx);

    Ok(state)
}

#[derive(Debug, Deserialize)]
#[serde(tag = "version", content = "state")]
enum LoadState {
    V0(StateV0),
}

impl LoadState {
    fn from_json_bytes(json: &[u8]) -> Result<Self> {
        let state = serde_json::from_slice::<Self>(json)
            .wrap_err("Error deserializing state")?;

        Ok(state)
    }

    async fn load_from_file() -> Result<Option<Self>> {
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

        let state = Self::from_json_bytes(&buf)?;

        Ok(Some(state))
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "version", content = "state")]
enum SaveState<'a> {
    V0(&'a StateV0),
}

impl SaveState<'_> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn load_state_v0_from_json() {
        let json = r#"{"version":"V0","state":{"testing_state":{}}}"#;

        let state =
            LoadState::from_json_bytes(json.as_bytes())
                .expect("Error loading state from JSON");

        assert!(matches!(state, LoadState::V0(_)));
    }
}

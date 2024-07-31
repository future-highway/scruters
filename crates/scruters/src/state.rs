//! State management for the application.
//!
//! The state is saved and loaded from the 'scruters.json'
//! file. It is designed to be version controlled.

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
    pub async fn load_from_file() -> Option<Self> {
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
                panic!(
                    "Permission denied reading file: {error:?}"
                );
            }
            Err(error) => {
                panic!("Error opening file: {error:?}");
            }
        };

        let mut file = file?;

        let mut buf = Vec::new();
        if let Err(error) = file.read_to_end(&mut buf).await
        {
            panic!("Error reading file: {error:?}");
        }

        let state = match serde_json::from_slice::<Self>(
            &buf,
        ) {
            Ok(state) => state,
            Err(error) => {
                panic!(
                    "Error deserializing file: {error:?}"
                );
            }
        };

        Some(state)
    }

    pub async fn save_to_file(&self) {
        let json = match serde_json::to_string_pretty(self)
        {
            Ok(json) => json,
            Err(error) => {
                panic!("Error serializing state: {error:?}")
            }
        };

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
                panic!(
                    "Permission denied writing file: {error:?}"
                );
            }
            Err(error) => {
                panic!("Error opening file: {error:?}");
            }
        };

        if let Err(error) =
            file.write_all(json.as_bytes()).await
        {
            panic!("Error writing file: {error:?}");
        }
    }
}

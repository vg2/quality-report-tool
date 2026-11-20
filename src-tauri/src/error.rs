use qrt_store::StoreError;
use serde::Serialize;

/// Error returned to the UI. Messages are user-facing and deliberately
/// generic: no file paths, SQL, or secret material crosses the IPC boundary.
/// Full details are logged on the Rust side.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
}

impl CommandError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_input",
            message: message.into(),
        }
    }
}

impl From<StoreError> for CommandError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::NotFound => Self {
                code: "not_found",
                message: "Not found.".into(),
            },
            other => {
                tracing::error!(error = %other, "storage error");
                Self {
                    code: "storage",
                    message: "Could not access workspace storage.".into(),
                }
            }
        }
    }
}

#[deny(missing_docs)]
/// `error` is a module that contains the error types for the AnkiConnect API.
pub mod error;
/// `notes` is a module that contains the note actions for the AnkiConnect API.
pub mod notes;
/// `result` is a module that contains the result types for the AnkiConnect API.
pub mod result;
/// `mock` is a module that contains mock implementations for the AnkiApi trait.
pub mod mock;
mod test;

use async_trait::async_trait;
use crate::error::AnkiError;
use crate::result::NotesInfoData;
use crate::notes::NoteAction;
use reqwest::Client;

#[cfg(feature = "mock")]
use mockall::automock;

/// `AnkiApi` is a trait that defines the interface for communicating with Anki.
#[cfg_attr(feature = "mock", automock)]
#[async_trait]
pub trait AnkiApi: Send + Sync {
    /// Finds note IDs based on a query.
    async fn find_note_ids(&self, query: &str) -> Result<Vec<u128>, AnkiError>;
    /// Retrieves information for the specified note IDs.
    async fn get_notes_infos(&self, ids: Vec<u128>) -> Result<Vec<NotesInfoData>, AnkiError>;
    /// Retrieves the names of all decks.
    async fn deck_names(&self) -> Result<Vec<String>, AnkiError>;
    /// Opens the specified note in the Anki browser for editing.
    async fn gui_edit_note(&self, id: u128) -> Result<(), AnkiError>;
}

/// `AnkiClient` is a struct that allows you to communicate with the AnkiConnect API.
///
/// It contains the following fields:
/// - `endpoint`: The endpoint where AnkiConnect is running. Defaults to `http://localhost:8765`.
/// - `client`: The HTTP client used to send requests.
/// - `version`: The version of the AnkiConnect plugin. Defaults to `6`.
#[derive(Clone, Debug)]
pub struct AnkiClient {
    pub endpoint: String,
    pub client: Client,
    pub version: u8,
}

impl Default for AnkiClient {
    /// Creates a new `AnkiClient` with default values.
    /// * `port`: The port where AnkiConnect is running. Defaults to `8765`.
    /// * `version`: The version of the AnkiConnect plugin. Defaults to `6`.
    ///
    /// To change these defaults, use `Ankiclient::new()` instead;
    ///
    /// # Example
    ///
    /// ```
    /// use anki_direct::AnkiClient;
    /// let client = AnkiClient::default();
    /// ```
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8765".to_string(),
            client: Client::new(),
            version: 6,
        }
    }
}

impl AnkiClient {
    /// Creates a new `AnkiClient` with the specified port and version.
    ///
    /// # Parameters
    ///
    /// * `port`: The port where AnkiConnect is running.
    /// * `version`: The version of the AnkiConnect plugin.
    ///
    /// # Example
    ///
    /// ```
    /// use anki_direct::AnkiClient;
    /// let client = AnkiClient::new("8765", 6);
    /// ```
    pub fn new(port: &str, version: u8) -> Self {
        Self {
            endpoint: format!("http://{port}"),
            client: Client::new(),
            version,
        }
    }

    /// Formats the URL from the provided port.
    ///
    /// # Parameters
    ///
    /// * `port`: The port where AnkiConnect is running.
    ///
    /// # Example
    ///
    /// ```
    /// use anki_direct::AnkiClient;
    /// let client = AnkiClient::default();
    /// let url = client.format_url("8765");
    /// ```
    pub fn format_url(&self, port: &str) -> String {
        format!("http://localhost:{port}")
    }
}

#[async_trait]
impl AnkiApi for AnkiClient {
    async fn find_note_ids(&self, query: &str) -> Result<Vec<u128>, AnkiError> {
        NoteAction::find_note_ids(self, query).await
    }

    async fn get_notes_infos(&self, ids: Vec<u128>) -> Result<Vec<NotesInfoData>, AnkiError> {
        NoteAction::get_notes_infos(self, ids).await
    }

    async fn deck_names(&self) -> Result<Vec<String>, AnkiError> {
        NoteAction::deck_names(self).await
    }

    async fn gui_edit_note(&self, id: u128) -> Result<(), AnkiError> {
        NoteAction::gui_edit_note(self, id).await
    }
}

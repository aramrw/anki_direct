mod notes;
mod result;
mod error;
mod test;

use reqwest::Client;

/// `AnkiClient` is a struct that allows you to communicate with the AnkiConnect API.
///
/// It contains the following fields:
/// - `endpoint`: The endpoint where AnkiConnect is running. Defaults to `http://localhost:8765`.
/// - `client`: The HTTP client used to send requests.
/// - `version`: The version of the AnkiConnect plugin. Defaults to `6`.
pub(crate) struct AnkiClient {
    endpoint: String,
    client: Client,
    version: u8,
}


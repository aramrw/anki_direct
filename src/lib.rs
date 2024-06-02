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

impl Default for AnkiClient {
    /// Creates a new `AnkiClient` with default values.
    /// * `port`: The port where AnkiConnect is running. Defaults to `8765`.
    /// * `version`: The version of the AnkiConnect plugin. Defaults to `6`.
    /// To change these defaults, use `Ankiclient::new()` instead;
    ///
    /// # Example
    ///
    /// ```
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
    /// let client = AnkiClient::new("8765", 6);
    /// ```
    fn new(port: &str, version: u8) -> Self {
        Self {
            endpoint: format!("http://{}", port),
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
    /// let url = client.format_url("8765");
    /// ```
    fn format_url(&self, port: &str) -> String {
        format!("http://localhost:{}", port)
    }

}

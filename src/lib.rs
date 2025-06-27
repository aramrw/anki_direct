pub mod anki;
pub mod error;
pub mod notes;
pub mod result;
mod str_utils;
mod test;
mod test_utils;

use std::ops::Deref;

use error::{AnkiError, CustomSerdeError};
use num_traits::PrimInt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

impl AnkiClient {
    /// Creates a new `AnkiClient` with the specified port.
    /// Returns an `Err(`[AnkiError::ConnectionNotFound]`)` if anki isn't open.
    ///
    /// # Parameters
    ///
    /// * `port`: The port where AnkiConnect is running.
    ///
    /// # Example
    ///
    /// ```
    /// // ankiconnect's default is "8765"
    /// let client = AnkiClient::new("8765");
    /// ```
    pub async fn new(port: &str) -> Result<Self, AnkiError> {
        let client = Client::new();
        let endpoint = Self::format_url(port);
        let version = AnkiClient::get_version_internal(&client, &endpoint).await?;
        let ac = Self {
            endpoint,
            client,
            version,
        };
        Ok(ac)
    }

    /// Creates a new `AnkiClient` with a port of "8765".
    /// This is the same as calling:
    /// ```
    /// let client = AnkiClient::new("8765");
    /// ```
    pub async fn default() -> Result<Self, AnkiError> {
        Self::new("8765").await
    }

    /// Sync fn that is the same as [Self::default()], except it also hardcodes the version as 6.
    ///
    /// `Note`: Not `async` because Doesn't check versions or make requests,
    /// so the first query could error if ankiconnect is not open, or the version is not `6`.
    pub fn default_latest_sync() -> Self {
        Self {
            endpoint: Self::format_url("8765"),
            client: Client::new(),
            version: 6,
        }
    }

    /// This fn is not `async` so you can initialize it in statics.
    ///
    /// `Note`: Not `async` because Doesn't check versions or make requests,
    /// so the first query could error if ankiconnect is not open, or the version is not correct.
    pub fn new_sync(port: &str, version: u8) -> Self {
        Self {
            endpoint: Self::format_url(port),
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
    pub fn format_url(port: &str) -> String {
        format!("http://localhost:{port}")
    }
    /// makes a get request to ankiconnect to get its version
    pub async fn get_version_internal(c: &Client, url: &str) -> Result<u8, AnkiError> {
        let res = match c.get(url).send().await {
            Ok(response) => response,
            Err(_) => return Err(AnkiError::ConnectionNotFound(url.to_string())),
        };
        let val: Value = res.json().await.unwrap();
        let Some(res) = val.as_object() else {
            let cse = CustomSerdeError::expected(None, val, None);
            return Err(AnkiError::CustomSerde(cse));
        };
        let version: String = res.get("apiVersion").unwrap().to_string();
        let mut version_str = version
            .split_once(".")
            .expect("no delimiter `.` found")
            .1
            .to_string();
        version_str.remove(1);
        let version = version_str
            .parse::<u8>()
            .map_err(|_| AnkiError::ParseIntError(version_str.to_string()))?;
        Ok(version)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Number(isize);
impl Number {
    pub fn new(int: impl PrimInt) -> Self {
        Self(
            int.to_isize()
                .unwrap_or_else(|| panic!("num cannot be converted to usize")),
        )
    }
    pub fn from_slice_to_vec(slice: &[impl PrimInt]) -> Vec<Number> {
        slice.iter().map(|int| Number::new(*int)).collect()
    }
}
impl Deref for Number {
    type Target = isize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#![allow(clippy::needless_doctest_main)]

pub mod anki;
#[cfg(feature = "cache")]
pub mod cache;
pub mod decks;
pub mod error;
mod generic;
pub mod model;
pub mod notes;
pub mod result;
mod str_utils;
mod test_utils;

use std::{ops::Deref, sync::Arc};

use error::{AnkiError, CustomSerdeError};
use getset::{Getters, MutGetters};
use num_traits::PrimInt;
use reqwest::blocking::Client as BlockingClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(feature = "cache")]
use crate::cache::Cache;
use crate::error::AnkiResult;
pub use reqwest::Client as ReqwestClient;

#[derive(Clone, Debug, Getters, MutGetters)]
pub struct AnkiClient {
    backend: Arc<Backend>,
    modules: Arc<AnkiModules>,
    #[cfg(feature = "cache")]
    #[getset(get = "pub", get_mut = "pub")]
    cache: Cache,
}
impl AnkiClient {
    /// Creates a new [AnkiClient] with the specified port.
    /// If `ankiconnect` isn't open\running on the port, returns `Err(`[AnkiError::ConnectionNotFound]`)`
    /// Has `_auto` prefix meaning it gets the version from [ankiconnect](https://git.sr.ht/~foosoft/anki-connect)
    /// # Parameters
    ///
    /// * `port`: The port where `ankiconnect` is running.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // ankiconnect's default is "8765"
    /// let client = Backend::new("8765");
    /// ```
    pub fn new_auto(port: &str) -> AnkiResult<Self> {
        let backend = Arc::new(Backend::new(port)?);
        let modules = Arc::new(AnkiModules::new(backend.clone()));
        Ok(Self {
            backend: backend.clone(),
            modules: modules.clone(),
            #[cfg(feature = "cache")]
            cache: Cache::init(modules),
        })
    }
    /// This fn is not `async` so you can initialize it in statics.
    ///
    /// `Note`: Not `async` because it doesn't check versions or make requests,
    /// so the first query could error if ankiconnect is not open, or the version is not correct.
    pub fn new_sync(port: &str, version: u8) -> Self {
        let backend = Arc::new(Backend::new_sync(port, version));
        let modules = Arc::new(AnkiModules::new(backend.clone()));
        Self {
            backend: backend.clone(),
            modules: modules.clone(),
            #[cfg(feature = "cache")]
            cache: Cache::init(modules),
        }
    }
    pub fn notes(&self) -> &NotesProxy {
        &self.modules.notes
    }
    pub fn models(&self) -> &ModelsProxy {
        &self.modules.models
    }
    /// Returns a reference to a reqwest client if needed.
    pub fn reqwest_client(&self) -> &BlockingClient {
        &self.backend.client
    }
}

#[derive(Clone, Debug, Getters)]
pub struct AnkiModules {
    backend: Arc<Backend>,
    notes: NotesProxy,
    models: ModelsProxy,
    #[getset(get = "pub")]
    decks: DecksProxy,
}
impl PartialEq for AnkiModules {
    fn eq(&self, other: &Self) -> bool {
        let Self { backend, .. } = other;
        self.backend == *backend
    }
}

impl AnkiModules {
    fn new(backend: Arc<Backend>) -> Self {
        Self {
            backend: backend.clone(),
            notes: NotesProxy(backend.clone()),
            models: ModelsProxy(backend.clone()),
            decks: DecksProxy(backend.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NotesProxy(Arc<Backend>);
impl Deref for NotesProxy {
    type Target = Arc<Backend>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Clone, Debug)]
pub struct ModelsProxy(Arc<Backend>);
impl Deref for ModelsProxy {
    type Target = Arc<Backend>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Clone, Debug)]
pub struct DecksProxy(Arc<Backend>);
impl Deref for DecksProxy {
    type Target = Arc<Backend>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for AnkiClient {
    fn default() -> Self {
        let backend = Arc::new(Backend::default());
        let modules = Arc::new(AnkiModules::new(backend.clone()));
        Self {
            backend: backend.clone(),
            modules: modules.clone(),
            #[cfg(feature = "cache")]
            cache: Cache::init(modules.clone()),
        }
    }
}
impl Deref for AnkiClient {
    type Target = Arc<Backend>;
    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

/// `Backend` is a struct that allows you to communicate with the AnkiConnect API.
///
/// It contains the following fields:
/// - `endpoint`: The endpoint where AnkiConnect is running. Defaults to `http://localhost:8765`.
/// - `client`: The HTTP client used to send requests.
/// - `version`: The version of the AnkiConnect plugin. Defaults to `6`.
#[derive(Clone, Debug)]
pub struct Backend {
    pub endpoint: String,
    pub client: BlockingClient,
    pub version: u8,
}

impl PartialEq for Backend {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            endpoint, version, ..
        } = self;
        other.endpoint == *endpoint && other.version == *version
    }
}
impl Eq for Backend {}

impl Default for Backend {
    /// Sync fn that is the same as [Self::default_auto()]
    /// except it also hardcodes the version as 6.
    ///
    /// `Note`: Not `async` because it doesn't check versions or make requests,
    /// so the first query could error if ankiconnect is not open, or the version is not `6`.
    fn default() -> Self {
        Self {
            endpoint: Self::format_url("8765"),
            client: BlockingClient::new(),
            version: 6,
        }
    }
}

impl Backend {
    /// Creates a new `Backend` with the specified port.
    /// Returns an `Err(`[AnkiError::ConnectionNotFound]`)` if anki isn't open.
    ///
    /// # Parameters
    ///
    /// * `port`: The port where AnkiConnect is running.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // ankiconnect's default is "8765"
    /// let client = Backend::new("8765");
    /// ```
    pub fn new(port: &str) -> Result<Self, AnkiError> {
        let client = BlockingClient::new();
        let endpoint = Self::format_url(port);
        let version = Backend::get_version_internal(&client, &endpoint)?;
        let ac = Self {
            endpoint,
            client,
            version,
        };
        Ok(ac)
    }

    /// Creates a new `Backend` with a port of "8765".
    /// This is the same as calling:
    /// ```
    /// let client = Backend::new("8765");
    /// ```
    pub fn default_auto() -> Result<Self, AnkiError> {
        Self::new("8765")
    }

    /// This fn is not `async` so you can initialize it in statics.
    ///
    /// `Note`: Not `async` because it doesn't check versions or make requests,
    /// so the first query could error if ankiconnect is not open, or the version is not correct.
    pub fn new_sync(port: &str, version: u8) -> Self {
        Self {
            endpoint: Self::format_url(port),
            client: BlockingClient::new(),
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
    pub fn get_version_internal(c: &BlockingClient, url: &str) -> Result<u8, AnkiError> {
        let res = match c.get(url).send() {
            Ok(response) => response,
            Err(_) => return Err(AnkiError::ConnectionNotFound(url.to_string())),
        };
        let val: Value = res.json().unwrap();
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

/// Abstraction over any non-floating point number
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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

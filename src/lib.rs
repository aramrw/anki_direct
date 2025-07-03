#![allow(clippy::needless_doctest_main)]

pub mod anki;
#[cfg(feature = "cache")]
pub mod cache;
pub mod decks;
pub mod error;
pub mod generic;
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
    /// Creates a new [AnkiClient] with the specified port, automatically detecting the AnkiConnect version.
    /// If `ankiconnect` isn't open or running on the port, returns `Err(`[AnkiError::ConnectionNotFound]`)`.
    ///
    /// # Parameters
    ///
    /// * `port`: The port where `ankiconnect` is running.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // AnkiConnect's default is "8765"
    /// let client = AnkiClient::new_port("8765");
    /// ```
    pub fn new_port(port: &str) -> AnkiResult<Self> {
        let backend = Arc::new(Backend::new_port(port)?);
        let modules = Arc::new(AnkiModules::new(backend.clone()));
        Ok(Self {
            backend: backend.clone(),
            modules: modules.clone(),
            #[cfg(feature = "cache")]
            cache: Cache::init(modules),
        })
    }

    /// Creates a new [AnkiClient] with the default port ("8765"), automatically detecting the AnkiConnect version.
    /// If `ankiconnect` isn't open or running, returns `Err(`[AnkiError::ConnectionNotFound]`)`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let client = AnkiClient::default_latest();
    /// ```
    pub fn default_latest() -> AnkiResult<Self> {
        let backend = Arc::new(Backend::default_latest()?);
        let modules = Arc::new(AnkiModules::new(backend.clone()));
        Ok(Self {
            backend: backend.clone(),
            modules: modules.clone(),
            #[cfg(feature = "cache")]
            cache: Cache::init(modules),
        })
    }

    /// Creates a new [AnkiClient] with the specified port and a hardcoded version.
    /// This function does not perform any checks for AnkiConnect availability or version compatibility.
    /// It is suitable for static initialization where the AnkiConnect instance is guaranteed to be running
    /// on the specified port and version.
    ///
    /// # Parameters
    ///
    /// * `port`: The port where `ankiconnect` is expected to be running.
    /// * `version`: The expected API version of AnkiConnect.
    ///
    /// # Example
    ///
    /// ```
    /// // Create an AnkiClient for AnkiConnect on port 8765 with API version 6
    /// let client = AnkiClient::new_port_version("8765", 6);
    /// ```
    pub fn new_port_version(port: &str, version: u8) -> Self {
        let backend = Arc::new(Backend::new_port_version(port, version));
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
    /// Creates a new `Backend` with the default port ("8765") and a hardcoded version of 6.
    /// This is a synchronous function and does not check for AnkiConnect availability or version compatibility.
    ///
    /// # Example
    ///
    /// ```
    /// let backend = Backend::default();
    /// ```
    fn default() -> Self {
        Self::new_port_version("8765", 6)
    }
}

impl Backend {
    /// Creates a new `Backend` with the specified port, automatically detecting the AnkiConnect version.
    /// Returns an `Err(`[AnkiError::ConnectionNotFound]`)` if AnkiConnect isn't open or reachable on the given port.
    ///
    /// # Parameters
    ///
    /// * `port`: The port where AnkiConnect is running.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // AnkiConnect's default port is "8765"
    /// let backend = Backend::new_port("8765");
    /// ```
    pub fn new_port(port: &str) -> Result<Self, AnkiError> {
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

    /// Creates a new `Backend` with the default port ("8765"), automatically detecting the AnkiConnect version.
    /// This is equivalent to calling `Backend::new_port("8765")`.
    /// Returns an `Err(`[AnkiError::ConnectionNotFound]`)` if AnkiConnect isn't open or reachable.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let backend = Backend::default_latest();
    /// ```
    pub fn default_latest() -> Result<Self, AnkiError> {
        Self::new_port("8765")
    }

    /// Creates a new `Backend` with the specified port and a hardcoded version.
    /// This function does not perform any checks for AnkiConnect availability or version compatibility.
    /// It is suitable for static initialization where the AnkiConnect instance is guaranteed to be running
    /// on the specified port and version.
    ///
    /// # Parameters
    ///
    /// * `port`: The port where AnkiConnect is expected to be running.
    /// * `version`: The expected API version of AnkiConnect.
    ///
    /// # Example
    ///
    /// ```
    /// // Create a backend for AnkiConnect on port 8765 with API version 6
    /// let backend = Backend::new_port_version("8765", 6);
    /// ```
    pub fn new_port_version(port: &str, version: u8) -> Self {
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

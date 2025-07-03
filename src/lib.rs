//! `anki_direct` is a Rust library for interacting with the AnkiConnect API.
//! It provides a convenient and type-safe way to manage your Anki collection,
//! including notes, models, and decks.
//!
//! The library aims to provide a comprehensive set of functionalities offered by AnkiConnect,
//! with a focus on ease of use and idiomatic Rust.
//!
//! # Features
//!
//! - **Notes Management**: Add, update, find, and delete notes.
//! - **Model Management**: Retrieve model information, including fields and templates.
//! - **Deck Management**: Get deck names and IDs.
//! - **Error Handling**: Robust error handling with custom error types.
//! - **Type-Safe API**: Strongly typed requests and responses for compile-time safety.
//! - **Blocking Client**: Uses `reqwest::blocking` for synchronous API calls.
//!
//! # Getting Started
//!
//! To use `anki_direct`, add it as a dependency in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! anki_direct = "0.0.1"
//! ```
//!
//! # Examples
//!
//! ## Creating an AnkiClient
//!
//! ```no_run
//! use anki_direct::AnkiClient;
//! use anki_direct::error::AnkiResult;
//!
//! fn main() -> AnkiResult<()> {
//!     // Connect to AnkiConnect on the default port (8765)
//!     let client = AnkiClient::default_latest()?;
//!     println!("Successfully connected to AnkiConnect!");
//!     Ok(())
//! }
//! ```
//!
//! ## Adding a Note
//!
//! ```no_run
//! use anki_direct::AnkiClient;
//! use anki_direct::notes::{NoteBuilder, MediaBuilder};
//! use anki_direct::error::AnkiResult;
//!
//! fn main() -> AnkiResult<()> {
//!     let client = AnkiClient::default_latest()?;
//!
//!     let audio = MediaBuilder::create_empty()
//!         .filename("example.mp3".into())
//!         .fields(vec!["myAudio".into()])
//!         .url("http://example.com/audio.mp3")
//!         .build()?;
//!
//!     let note = NoteBuilder::create_empty()
//!         .model_name("Basic".into())
//!         .deck_name("Default".into())
//!         .field("Front", "Hello")
//!         .field("Back", "World")
//!         .audios(vec![audio])
//!         .build(Some(client.reqwest_client()))?;
//!
//!     let new_ids = client.notes().add_notes(&[note])?;
//!     println!("Added note with ID: {:?}", new_ids);
//!     Ok(())
//! }
//! ```

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

/// `AnkiClient` is the primary entry point for interacting with the AnkiConnect API.
/// It manages the connection to AnkiConnect and provides access to various modules
/// for managing notes, models, and decks.
///
/// # Examples
///
/// ```no_run
/// use anki_direct::AnkiClient;
/// use anki_direct::error::AnkiResult;
///
/// fn main() -> AnkiResult<()> {
///     let client = AnkiClient::default_latest()?;
///     println!("Successfully connected to AnkiConnect!");
///     Ok(())
/// }
/// ```
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

    /// Provides access to notes-related AnkiConnect API calls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anki_direct::AnkiClient;
    /// use anki_direct::anki::AnkiQuery;
    /// use anki_direct::error::AnkiResult;
    ///
    /// fn main() -> AnkiResult<()> {
    ///     let client = AnkiClient::default_latest()?;
    ///     let new_notes = client.notes().find_notes(AnkiQuery::from("is:new"))?;
    ///     println!("Found {} new notes.", new_notes.len());
    ///     Ok(())
    /// }
    /// ```
    pub fn notes(&self) -> &NotesProxy {
        &self.modules.notes
    }

    /// Provides access to model-related AnkiConnect API calls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anki_direct::AnkiClient;
    /// use anki_direct::error::AnkiResult;
    ///
    /// fn main() -> AnkiResult<()> {
    ///     let client = AnkiClient::default_latest()?;
    ///     let models = client.models().get_all_models_less()?;
    ///     for (name, details) in models {
    ///         println!("Model: {}, Fields: {:?}", name, details.fields);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn models(&self) -> &ModelsProxy {
        &self.modules.models
    }

    /// Provides access to deck-related AnkiConnect API calls.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anki_direct::AnkiClient;
    /// use anki_direct::error::AnkiResult;
    ///
    /// fn main() -> AnkiResult<()> {
    ///     let client = AnkiClient::default_latest()?;
    ///     let decks = client.decks().get_all_deck_names_and_ids()?;
    ///     for (name, id) in decks {
    ///         println!("Deck: {}, ID: {}", name, id);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn decks(&self) -> &DecksProxy {
        &self.modules.decks
    }

    /// Returns a reference to the internal `reqwest::blocking::Client` used by `anki_direct`.
    /// This can be useful if you need to perform custom HTTP requests to AnkiConnect
    /// or other services using the same client configuration.
    pub fn reqwest_client(&self) -> &BlockingClient {
        &self.backend.client
    }
}

/// `AnkiModules` is an internal struct that holds references to the various API modules
/// (notes, models, decks) and the shared `Backend`.
/// It's primarily used internally by `AnkiClient` to organize and provide access to
/// different parts of the AnkiConnect API.
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

/// `NotesProxy` provides methods for interacting with notes in Anki.
/// It's a thin wrapper around the `Backend` that exposes notes-related AnkiConnect API calls.
/// You can access this through `AnkiClient::notes()`.
#[derive(Clone, Debug)]
pub struct NotesProxy(Arc<Backend>);
impl Deref for NotesProxy {
    type Target = Arc<Backend>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// `ModelsProxy` provides methods for interacting with models (note types) in Anki.
/// It's a thin wrapper around the `Backend` that exposes model-related AnkiConnect API calls.
/// You can access this through `AnkiClient::models()`.
#[derive(Clone, Debug)]
pub struct ModelsProxy(Arc<Backend>);
impl Deref for ModelsProxy {
    type Target = Arc<Backend>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// `DecksProxy` provides methods for interacting with decks in Anki.
/// It's a thin wrapper around the `Backend` that exposes deck-related AnkiConnect API calls.
/// You can access this through `AnkiClient::decks()`.
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
/// It handles the underlying HTTP requests and response parsing.
///
/// It contains the following fields:
/// - `endpoint`: The URL where AnkiConnect is running. Defaults to `http://localhost:8765`.
/// - `client`: The HTTP client used to send requests (`reqwest::blocking::Client`).
/// - `version`: The API version of the AnkiConnect plugin that the backend is configured to use.
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
    /// use anki_direct::Backend;
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
    /// use anki_direct::Backend;
    /// use anki_direct::error::AnkiResult;
    ///
    /// fn main() -> AnkiResult<()> {
    ///     // AnkiConnect's default port is "8765"
    ///     let backend = Backend::new_port("8765")?;
    ///     Ok(())
    /// }
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
    /// use anki_direct::Backend;
    /// use anki_direct::error::AnkiResult;
    ///
    /// fn main() -> AnkiResult<()> {
    ///     let backend = Backend::default_latest()?;
    ///     Ok(())
    /// }
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
    /// use anki_direct::Backend;
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
    /// use anki_direct::Backend;
    /// let url = Backend::format_url("8765");
    /// ```
    pub fn format_url(port: &str) -> String {
        format!("http://localhost:{port}")
    }

    /// Makes a GET request to AnkiConnect to retrieve its version.
    /// This is an internal helper function used during backend initialization.
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

/// A wrapper struct for integer types, providing a consistent way to handle numbers
/// that can be used as IDs or counts in AnkiConnect API calls.
/// It dereferences to an `isize`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Number(isize);
impl Number {
    /// Creates a new `Number` from any primitive integer type.
    /// Panics if the conversion to `isize` fails (e.g., for very large `u128`).
    pub fn new(int: impl PrimInt) -> Self {
        Self(
            int.to_isize()
                .unwrap_or_else(|| panic!("num cannot be converted to isize")),
        )
    }

    /// Converts a slice of primitive integers into a vector of `Number`.
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

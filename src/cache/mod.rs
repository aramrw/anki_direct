//! Caching functionality for `anki_direct`.
//!
//! # Serialization and Hydration
//!
//! The cache structs (`Cache`, `ModelCache`, `DeckCache`) are designed to be serializable,
//! allowing you to persist fetched data to disk. However, a critical aspect of this
//! feature is the concept of "hydration".
//!
//! The live connection to the AnkiConnect backend (via `reqwest::blocking::Client`)
//! is **not** serializable. Therefore, this connection is skipped during the serialization
//! process (using `#[serde(skip)]`).
//!
//! When you deserialize a cache, it loads all the stored data (like model and deck info)
//! but does **not** have a live connection to Anki. In this state, the cache is considered
//! "dehydrated". You can still access the data within it, but you cannot perform any
//! operations that require fetching new data from Anki (e.g., `update_all`, `hydrate`).
//!
//! To make a deserialized cache fully operational again, you must **re-hydrate** it by
//! providing a new, live connection from an `AnkiClient` instance.
//!
//! See the documentation for [`Cache`] for a detailed example of the
//! serialize-deserialize-hydrate workflow.

#![allow(dead_code)]
pub mod deck;
pub mod model;

use crate::{
    cache::{deck::DeckCache, model::ModelCache},
    error::AnkiResult,
    AnkiModules,
};
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

type Mod = Option<Arc<AnkiModules>>;

#[derive(Error, Debug)]
pub enum CacheError {
    /// Error indicating that an operation requiring a live AnkiConnect connection was attempted
    /// on a dehydrated cache.
    ///
    /// A cache becomes "dehydrated" after being deserialized because the live connection
    /// (containing a non-serializable `reqwest::blocking::Client`) is skipped during serialization.
    /// To fix this, the cache must be "re-hydrated" with a new connection.
    ///
    /// **See:** [`Cache::hydrate`] for the solution.
    #[error("Operation requires a live connection, but the cache is dehydrated.\n\n[reason]: The backend connection is always removed during serialization.\n[help]: Call \"Cache::hydrate(..)\" to re-connect to ankiconnect")]
    Dehydrated,
}

/// Anki Cache convenience wrapper.
///
/// This provides a default cache implementation that stores model and deck details.
///
/// # Workflow: Serialization and Hydration
///
/// The main purpose of the cache is to store fetched Anki data, so you don't have to
/// retrieve it every time your application starts. The intended workflow is:
///
/// 1.  **Fetch and Cache:** Create an `AnkiClient`, fetch data, and populate the cache.
/// 2.  **Serialize:** Serialize the `Cache` object (e.g., to a file using `bincode`).
///     The live connection is automatically skipped.
/// 3.  **Shutdown:** Your application closes.
/// 4.  **Relaunch & Deserialize:** On next launch, deserialize the `Cache` from the file.
///     It now contains all your data but is "dehydrated" (no live connection).
/// 5.  **Re-hydrate:** Create a new `AnkiClient` to establish a fresh connection, then
///     use its modules to `hydrate` the deserialized cache.
///
/// The cache is now fully operational again, with its old data intact and the ability
/// to fetch new data from Anki.
///
/// # Example
///
/// ```no_run
/// use anki_direct::AnkiClient;
/// use anki_direct::error::AnkiResult;
/// use std::fs::File;
/// use std::sync::Arc;
///
/// fn manage_cache() -> AnkiResult<()> {
///     let client = AnkiClient::default_latest()?;
///     let cache_path = "my_anki_cache.bin";
///
///     // Try to load the cache from a file
///     let mut cache = if let Ok(file) = File::open(cache_path) {
///         println!("Loading cache from disk...");
///         let mut deserialized_cache: anki_direct::cache::Cache = bincode::deserialize_from(file).unwrap();
///
///         // Re-hydrate the cache with a new, live connection
///         println!("Re-hydrating cache...");
///         deserialized_cache.hydrate(client.modules().clone());
///         deserialized_cache
///     } else {
///         // No cache file found, create a new one
///         println!("No cache found. Creating a new one.");
///         client.cache().clone()
///     };
///
///     // Use the cache, and update it with the latest from Anki
///     println!("Updating cache with latest data from Anki...");
///     cache.update_all()?;
///     println!("Models in cache: {}", cache.models().len());
///
///     // Serialize the updated cache to disk for next time
///     println!("Saving cache to disk...");
///     let file = File::create(cache_path)?;
///     bincode::serialize_into(file, &cache).unwrap();
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Getters, Setters, MutGetters)]
pub struct Cache {
    #[serde(skip)]
    modules: Mod,
    models: ModelCache<String>,
    #[getset(get = "pub", get_mut = "pub")]
    decks: DeckCache<String>,
}

impl Cache {
    /// Initializes a new, empty cache with a live connection.
    ///
    /// This is typically called internally when a new `AnkiClient` is created.
    pub fn init(modules: Arc<AnkiModules>) -> Self {
        Self {
            modules: modules.clone().into(),
            models: ModelCache::new(modules.clone()),
            decks: DeckCache::new(modules.clone()),
        }
    }

    /// Returns a reference to the model cache.
    pub fn models(&self) -> &ModelCache<String> {
        &self.models
    }

    /// Returns a mutable reference to the model cache.
    pub fn models_mut(&mut self) -> &mut ModelCache<String> {
        &mut self.models
    }

    /// Fetches the latest data for all managed caches from Anki.
    ///
    /// This operation requires the cache to be "hydrated" (i.e., have a live connection).
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Dehydrated`] if called on a deserialized cache that has not
    /// been re-hydrated.
    pub fn update_all(&mut self) -> AnkiResult<()> {
        self.models_mut().hydrate()?;
        Ok(())
    }

    /// Re-hydrates the cache with a new, live connection to Anki.
    ///
    /// This method is essential for making a deserialized cache fully operational again.
    /// It takes the `AnkiModules` from a new `AnkiClient` instance and attaches them
    /// to the cache, enabling it to perform actions that require a connection.
    ///
    /// # Parameters
    ///
    /// - `modules`: An `Arc<AnkiModules>` from a live `AnkiClient`.
    pub fn hydrate(&mut self, modules: Arc<AnkiModules>) -> &mut Self {
        self.modules = Some(modules);
        self
    }
}

#[cfg(test)]
mod cache {
    use std::sync::LazyLock;

    use bincode::config::Configuration;

    use crate::AnkiClient;

    static BINCODECONFIG: LazyLock<Configuration> = LazyLock::new(|| {
        bincode::config::standard()
            .with_little_endian()
            .with_variable_int_encoding()
    });

    #[ignore]
    #[test]
    fn bincode() {
        let mut ankiclient = AnkiClient::default_latest().unwrap();
        let cache = ankiclient.cache_mut();
        cache.update_all();
        let note_models = cache.models().clone();
        //bincode::encode_into_std_write(note_models, &BINCODECONFIG);
    }
}

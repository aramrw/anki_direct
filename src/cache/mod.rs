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
    #[error("Operation requires a live connection, but the cache is dehydrated.\n\n[reason]: The backend connection is always removed during serialization.\n[help]: Call \"Cache::hydrate_all(..)\" to re-connect to ankiconnect")]
    Dehydrated,
}

/// Anki Cache convenience wrapper.
///
/// This provides a default cache implementation that stores model details
/// keyed by their name (String).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Getters, Setters, MutGetters)]
pub struct Cache {
    #[serde(skip)]
    modules: Mod,
    models: ModelCache<String>,
    #[getset(get = "pub", get_mut = "pub")]
    decks: DeckCache<String>,
}

impl Cache {
    /// Initializes the default cache.
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
    pub fn update_all(&mut self) -> AnkiResult<()> {
        self.models_mut().hydrate()?;
        Ok(())
    }

    /// Rehydrates [Cache] with a new client connection.
    pub fn hydrate(&mut self, modules: Arc<AnkiModules>) -> &mut Self {
        self.modules = Some(modules);
        self
    }
}

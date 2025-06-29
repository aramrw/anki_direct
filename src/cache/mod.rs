#![allow(dead_code)]
pub mod modelcache;

use crate::{
    error::{AnkiError, AnkiResult},
    model::FullModelDetails,
    AnkiModules,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, hash::Hash, iter::Map, ops::Deref, sync::Arc};
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cache {
    #[serde(skip)]
    modules: Mod,
    models: ModelCache<String>,
}

impl Cache {
    /// Initializes the default cache.
    pub fn init(modules: Arc<AnkiModules>) -> Self {
        Self {
            modules: modules.clone().into(),
            models: ModelCache::new(modules.clone()),
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
    pub async fn update_all(&mut self) -> AnkiResult<()> {
        self.models_mut().hydrate().await?;
        Ok(())
    }

    /// Rehydrates [Cache] with a new client connection.
    pub fn hydrate(&mut self, modules: Arc<AnkiModules>) -> &mut Self {
        self.modules = Some(modules);
        self
    }
}

/// A generic cache for Anki models, allowing the user to specify the key type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelCache<K>
where
    K: Hash + Eq,
{
    #[serde(skip)]
    modules: Mod,
    cache: IndexMap<K, FullModelDetails>,
}

/// General implementation for any key type `K`.
impl<K> ModelCache<K>
where
    K: Hash + Eq,
{
    /// Creates a new, empty model cache.
    pub fn new(modules: Arc<AnkiModules>) -> Self {
        Self {
            modules: modules.into(),
            cache: IndexMap::new(),
        }
    }
}

/// This implementation is only available when the key `K` is a `String`.
/// It provides the `update` method which fetches data from the AnkiConnect API.
impl ModelCache<String> {
    /// Hydrates [ModelCache] to use latest models from `Anki`.
    /// The existing data in the cache will be replaced.
    pub async fn hydrate(&mut self) -> AnkiResult<&mut Self> {
        let Some(modules) = &self.modules else {
            return Err(AnkiError::Cache(CacheError::Dehydrated));
        };
        let latest: IndexMap<String, FullModelDetails> =
            modules.models.get_all_models_full().await?;

        self.cache = latest;
        Ok(self)
    }
}

/// Helper functions for caches with clonable keys.
impl<K> ModelCache<K>
where
    K: Hash + Eq,
{
    /// Finds multiple models by their keys and returns owned copies of the keys and values.
    ///
    /// This is useful for when you want to search with `&str` but get back `(String, FullModelDetails)`.
    pub fn find_many_from_key_owned<'a, Q>(
        &'a self,
        keys: &'a [&Q],
    ) -> impl Iterator<Item = (K, FullModelDetails)> + 'a
    where
        K: Borrow<Q> + Clone,
        Q: Hash + Eq + ?Sized,
    {
        keys.iter()
            .filter_map(move |key| self.get_key_value(*key))
            .map(|(k, v)| (k.clone(), v.clone()))
    }

    pub fn get_cache(&self) -> &IndexMap<K, FullModelDetails> {
        &self.cache
    }
}

/// Allows read-only access to the underlying `IndexMap` of the cache.
impl<K> Deref for ModelCache<K>
where
    K: Hash + Eq,
{
    type Target = IndexMap<K, FullModelDetails>;
    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

impl<T: Eq + Hash> From<ModelCache<T>> for IndexMap<T, FullModelDetails> {
    fn from(val: ModelCache<T>) -> Self {
        val.cache
    }
}

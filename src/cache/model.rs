#![allow(dead_code)]

use crate::{
    cache::{CacheError, Mod},
    error::{AnkiError, AnkiResult},
    model::FullModelDetails,
    AnkiModules,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, hash::Hash, ops::Deref, sync::Arc};

/// A generic cache for Anki models, allowing the user to specify the key type.
///
/// # Serialization
///
/// This struct can be serialized. However, the `modules` field, which holds the live
/// connection to Anki, is skipped during serialization (`#[serde(skip)]`).
///
/// After deserializing, the cache will be "dehydrated" and must be re-hydrated with a
/// new connection to perform any operations that fetch data from Anki.
///
/// For a complete explanation and example of the serialization/hydration workflow,
/// please see the main [`cache` module documentation](super).
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
    /// Creates a new, empty model cache with a live connection.
    pub fn new(modules: Arc<AnkiModules>) -> Self {
        Self {
            modules: modules.into(),
            cache: IndexMap::new(),
        }
    }
}

/// This implementation is only available when the key `K` is a `String`.
/// It provides methods that fetch data from the AnkiConnect API.
impl ModelCache<String> {
    /// Fetches all models from Anki and replaces the existing cache with the latest data.
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Dehydrated`](super::CacheError::Dehydrated) if the cache does not have a live connection.
    pub fn hydrate(&mut self) -> AnkiResult<&mut Self> {
        let Some(modules) = &self.modules else {
            return Err(AnkiError::Cache(CacheError::Dehydrated));
        };
        let latest: IndexMap<String, FullModelDetails> = modules.models.get_all_models_full()?;

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

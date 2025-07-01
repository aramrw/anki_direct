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
use thiserror::Error;

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

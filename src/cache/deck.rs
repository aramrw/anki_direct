use std::{borrow::Borrow, hash::Hash, ops::Deref, sync::Arc};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    cache::{CacheError, Mod},
    decks::DeckConfig,
    error::{AnkiError, AnkiResult},
    AnkiModules, Number,
};

/// A cache for Anki deck configurations, keyed by a generic type `K`.
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
pub struct DeckCache<K>
where
    K: Hash + Eq,
{
    #[serde(skip)]
    modules: Mod,
    cache: IndexMap<K, Option<DeckConfig>>,
}

/// General implementation for any key type `K`.
impl<K> DeckCache<K>
where
    K: Hash + Eq,
{
    /// Creates a new, empty deck cache with a live connection.
    pub fn new(modules: Arc<AnkiModules>) -> Self {
        Self {
            modules: modules.into(),
            cache: IndexMap::new(),
        }
    }
}

/// This implementation is only available when the key `K` is a `String`.
/// It provides methods that fetch data from the AnkiConnect API.
impl DeckCache<String> {
    /// Fetches all deck names from Anki and adds any new ones to the cache.
    ///
    /// Existing deck configurations in the cache are left untouched. This is useful when
    /// you only need to be aware of new decks (e.g., for creating notes) without
    /// fetching detailed configuration for all decks.
    ///
    /// # Errors
    ///
    /// Returns [`CacheError::Dehydrated`](super::CacheError::Dehydrated) if the cache does not have a live connection.
    pub fn hydrate_names(&mut self) -> AnkiResult<&mut Self> {
        let Some(modules) = &self.modules else {
            return Err(AnkiError::Cache(CacheError::Dehydrated));
        };
        let latest: IndexMap<String, Number> = modules.decks.get_all_deck_names_and_ids()?;
        for (name, _) in latest {
            if !self.cache.contains_key(&name) {
                self.cache.insert(name, None);
            } else {
                // ignore existing entries, as we have no new info for it
            }
        }

        Ok(self)
    }
}

/// Helper functions for caches with clonable keys.
impl<K> DeckCache<K>
where
    K: Hash + Eq,
{
    /// Finds multiple models by their keys and returns owned copies of the keys and values.
    ///
    /// This is useful for when you want to search with `&str` but get back `(String, FullModelDetails)`.
    pub fn find_many_from_key_owned<'a, Q>(
        &'a self,
        keys: &'a [&Q],
    ) -> impl Iterator<Item = (K, Option<DeckConfig>)> + 'a
    where
        K: Borrow<Q> + Clone,
        Q: Hash + Eq + ?Sized,
    {
        keys.iter()
            .filter_map(move |key| self.get_key_value(*key))
            .map(|(k, v)| (k.clone(), v.clone()))
    }
    pub fn get_cache(&self) -> &IndexMap<K, Option<DeckConfig>> {
        &self.cache
    }
    /// Moves cache out of self and returns it
    pub fn take_cache(self) -> IndexMap<K, Option<DeckConfig>> {
        self.cache
    }
}

/// Allows read-only access to the underlying `IndexMap` of the cache.
impl<K> Deref for DeckCache<K>
where
    K: Hash + Eq,
{
    type Target = IndexMap<K, Option<DeckConfig>>;
    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

impl<T: Eq + Hash> From<DeckCache<T>> for IndexMap<T, Option<DeckConfig>> {
    fn from(val: DeckCache<T>) -> Self {
        val.cache
    }
}

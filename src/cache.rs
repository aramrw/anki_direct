#![allow(dead_code)]

use crate::{error::AnkiResult, model::LessModelDetails, AnkiModules};
use indexmap::IndexMap;
use std::sync::Arc;

/// Anki Cache
#[derive(Debug, Clone)]
pub struct Cache {
    modules: Arc<AnkiModules>,
    models: ModelCache,
}

impl Cache {
    pub fn init(modules: Arc<AnkiModules>) -> Self {
        Self {
            modules: modules.clone(),
            models: ModelCache::new(modules.clone()),
        }
    }
    pub fn models(&self) -> &ModelCache {
        &self.models
    }
    pub fn models_mut(&mut self) -> &mut ModelCache {
        &mut self.models
    }
    pub async fn update_all(&mut self) -> AnkiResult<()> {
        self.models_mut().update().await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ModelCache {
    modules: Arc<AnkiModules>,
    cache: IndexMap<String, LessModelDetails>,
}
impl ModelCache {
    pub fn new(modules: Arc<AnkiModules>) -> Self {
        Self {
            modules,
            cache: IndexMap::default(),
        }
    }
    pub async fn update(&mut self) -> AnkiResult<()> {
        let latest = self.modules.models.get_all_models_less().await?;
        self.cache = latest;
        Ok(())
    }
    fn get_cache(&self) -> &IndexMap<String, LessModelDetails> {
        &self.cache
    }
}

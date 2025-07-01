use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    error::AnkiResult,
    generic::{GenericRequest, GenericRequestBuilder},
    DecksProxy, Number,
};

/// https://git.sr.ht/~foosoft/anki-connect#codegetdeckconfigcode
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeckConfig {
    id: Number,
    name: String,
}

impl DecksProxy {
    pub async fn get_all_deck_names_and_ids(&self) -> AnkiResult<IndexMap<String, Number>> {
        type DecksResult = IndexMap<String, Number>;
        let payload: GenericRequest<()> = GenericRequestBuilder::default()
            .action("deckNamesAndIds".into())
            .version(self.version)
            .build()?;
        self.post_generic_request::<DecksResult>(payload).await
    }
}

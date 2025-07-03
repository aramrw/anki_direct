use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    error::AnkiResult,
    generic::{GenericRequest, GenericRequestBuilder},
    DecksProxy, Number,
};

/// `DeckConfig` represents the configuration of a single Anki deck.
/// It contains the deck's unique ID and its name.
///
/// <https://git.sr.ht/~foosoft/anki-connect#codegetdeckconfigcode>
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeckConfig {
    id: Number,
    name: String,
}

impl DecksProxy {
    /// Retrieves a map of all deck names to their corresponding IDs in the Anki collection.
    ///
    /// This function makes an AnkiConnect API call to `deckNamesAndIds`.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(IndexMap<String, Number>)`: A map where keys are deck names (String) and values are their IDs (Number).
    /// - `Err(AnkiError)`: If there was an error communicating with AnkiConnect or parsing the response.
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
    pub fn get_all_deck_names_and_ids(&self) -> AnkiResult<IndexMap<String, Number>> {
        type DecksResult = IndexMap<String, Number>;
        let payload: GenericRequest<()> = GenericRequestBuilder::default()
            .action("deckNamesAndIds".into())
            .version(self.version)
            .build()?;
        self.post_generic_request::<DecksResult>(payload)
    }
}

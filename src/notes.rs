#![allow(non_snake_case)]
use crate::anki::{AnkiQuery, GenericResult};
use crate::error::{AnkiError, CustomSerdeError};
use crate::result::NotesInfoData;
use crate::{AnkiClient, Number};
use num_traits::PrimInt;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub id: u128,
    pub fields: HashMap<String, String>,
    pub audio: Vec<Media>,
    pub picture: Option<Vec<Media>>,
}

#[derive(Serialize, Deserialize)]
pub struct Media {
    pub url: String,
    pub filename: String,
    pub skipHash: Option<String>,
    pub fields: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GuiEditNoteParams {
    pub note: Number,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateNoteParams {
    pub note: Note,
}

#[derive(Serialize, Deserialize)]
pub struct FindNotesParams {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct NotesInfoParams {
    pub notes: Vec<Number>,
}

// other
#[derive(Serialize, Deserialize)]
pub struct ConfigJson {
    pub fields: UserNoteFields,
}

#[derive(Serialize, Deserialize)]
pub struct UserNoteFields {
    pub expression: String,
    pub sentence: String,
    pub sentence_audio: String,
    pub image: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Params {
    UpdateNote(UpdateNoteParams),
    FindNotes(FindNotesParams),
    NotesInfo(NotesInfoParams),
    GuiEditNote(GuiEditNoteParams),
}

/// Creates a note using the given deck and model, with the provided field values and tags.
/// # Returns
/// the identifier of the created note created on success, or an err on failure.
#[derive(Serialize, Deserialize)]
pub struct NoteAction<'a> {
    pub action: &'a str,
    pub version: u8,
    pub params: Params,
}

impl AnkiClient {
    /// Returns note ids from a given query
    ///
    /// # Usage
    ///
    /// ```no_run
    /// let res = ANKICLIENT
    ///    .find_note_ids(AnkiQuery::CardState(CardState::IsNew))
    ///    .await?;
    /// ```
    ///
    /// # Example Result
    /// ```no_run
    /// {
    ///    "result": [1483959289817, 1483959291695],
    ///    "error": null
    /// }
    /// ```
    pub async fn find_note_ids(&self, query: AnkiQuery) -> Result<Vec<isize>, AnkiError> {
        let payload = NoteAction {
            action: "findNotes",
            version: self.version,
            params: Params::FindNotes(FindNotesParams {
                query: query.to_string(),
            }),
        };
        self.post_generic_request::<Vec<isize>>(payload).await
    }

    pub async fn get_notes_infos(
        &self,
        ids: &[impl PrimInt],
    ) -> Result<Vec<NotesInfoData>, AnkiError> {
        let payload = NoteAction {
            action: "notesInfo",
            version: self.version,
            params: Params::NotesInfo(NotesInfoParams {
                notes: Number::from_slice_to_vec(ids),
            }),
        };
        let res = self
            .post_generic_request::<Vec<NotesInfoData>>(payload)
            .await?;
        if res.is_empty() {
            return Err(AnkiError::NoDataFound);
        }
        Ok(res)
    }

    pub async fn gui_edit_note(&self, id: impl PrimInt) -> Result<(), AnkiError> {
        let payload = NoteAction {
            action: "guiEditNote",
            version: 6,
            params: {
                Params::GuiEditNote(GuiEditNoteParams {
                    note: Number::new(id),
                })
            },
        };
        self.post_generic_request::<()>(payload).await?;
        Ok(())
    }
}

impl AnkiClient {
    /// internal generic request
    /// `<T>` specifies the `result` field for [GenericResult]
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let payload = NoteAction {..};
    /// let res: Result<Vec<isize>> = self.post_generic_request::<Vec<isize>>(payload).await
    /// ```
    async fn post_generic_request<T: DeserializeOwned + Debug>(
        &self,
        payload: impl Serialize,
    ) -> Result<T, AnkiError> {
        let (client, endpoint) = (&self.client, &self.endpoint);
        let res = match client.post(endpoint).json(&payload).send().await {
            Ok(response) => response,
            Err(e) => return Err(AnkiError::RequestError(e.to_string())),
        };

        let mut val: Value = res.json().await?;
        if let Some(result_array) = val.get_mut("result").and_then(|r| r.as_array_mut()) {
            result_array.retain(|item| {
                match item.as_object() {
                    // only keep if not empty
                    Some(obj) => !obj.is_empty(),
                    // always keep if not an object
                    None => true,
                }
            });
        }
        let body: GenericResult<T> = serde_json::from_value(val.clone()).map_err(|e| {
            let cse = CustomSerdeError::expected(
                Some(crate::test_utils::display_type::<GenericResult<T>>()),
                val,
                Some(e.to_string()),
            );
            AnkiError::CustomSerde(cse)
        })?;
        if let Some(err) = body.error {
            return Err(AnkiError::AnkiConnect(err));
        }
        Ok(body.result)
    }
}
#[cfg(test)]
mod note_tests {
    use crate::{
        anki::{AnkiQuery, CardState},
        test_utils::ANKICLIENT,
    };

    #[tokio::test]
    async fn find_new_note_ids() {
        let res = ANKICLIENT
            .find_note_ids(AnkiQuery::CardState(CardState::IsNew))
            .await
            .map_err(|e| e.pretty_panic(file!(), line!()))
            .unwrap();
        assert!(!res.is_empty());
        let res = ANKICLIENT
            .find_note_ids(AnkiQuery::CardState(CardState::IsLearn))
            .await
            .map_err(|e| e.pretty_panic(file!(), line!()))
            .unwrap();
        assert!(!res.is_empty())
    }

    #[tokio::test]
    async fn get_notes_infos() {
        ANKICLIENT
            .get_notes_infos(&[12345])
            .await
            .map_err(|e| e.pretty_panic(file!(), line!()))
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn gui_edit_note() {
        ANKICLIENT
            .gui_edit_note(1749686091185 as usize)
            .await
            .map_err(|e| e.pretty_panic(file!(), line!()))
            .unwrap();
    }
}

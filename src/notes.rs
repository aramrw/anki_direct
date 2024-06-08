#![allow(non_snake_case)]
use crate::error::format_error;
use crate::result::NumVecRes;
use crate::error::{format_error, AnkiError};
use crate::result::{NotesInfoData, NotesInfoRes, NumVecRes};
use crate::AnkiClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Note {
    id: u64,
    fields: HashMap<String, String>,
    audio: Vec<Media>,
    picture: Option<Vec<Media>>,
}

#[derive(Serialize, Deserialize)]
struct Media {
    url: String,
    filename: String,
    skipHash: Option<String>,
    fields: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct UpdateNoteParams {
    note: Note,
}

#[derive(Serialize, Deserialize)]
struct FindNotesParams {
    query: String,
}

#[derive(Serialize, Deserialize)]
struct NotesInfoParams {
    notes: Vec<u64>,
}

// other
#[derive(Serialize, Deserialize)]
struct ConfigJson {
    fields: UserNoteFields,
}

#[derive(Serialize, Deserialize)]
struct UserNoteFields {
    pub expression: String,
    pub sentence: String,
    pub sentence_audio: String,
    pub image: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Params {
    UpdateNoteParams(UpdateNoteParams),
    FindNotesParams(FindNotesParams),
}

#[derive(Serialize, Deserialize)]
pub struct NoteAction {
    action: String,
    version: u8,
    params: Params,
}

impl NoteAction {
    pub async fn find_note_ids(
        anki_client: &AnkiClient,
        query: &str,
    ) -> Result<Option<Vec<u64>>, String> {
        let payload = NoteAction {
            action: "findNotes".to_string(),
            version: anki_client.version,
            params: Params::FindNotesParams(FindNotesParams {
                query: query.to_string(),
            }),
        };

        post_request(payload, &anki_client.endpoint, &anki_client.client).await
    }
}

async fn post_request(
    payload: NoteAction,
    endpoint: &str,
    client: &Client,
) -> Result<Option<Vec<u64>>, String> {
    let res: Result<NumVecRes, String> = client
        .post(endpoint)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format_error("Find Notes -> Network Err", e.to_string()))?
        .json()
        .await
        .map_err(|e| format_error("Find Notes -> Network Err", e.to_string()))?;

    let res = match res {
        Ok(res) => res,
        Err(err) => return Err(format_error("Find Notes -> Network Err", err.to_string())),
    };

    res.into_result()
}

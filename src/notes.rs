#![allow(non_snake_case)]
use crate::error::{format_error, AnkiError};
use crate::result::{NotesInfoData, NotesInfoRes, NumVecRes};
use crate::AnkiClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
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
pub struct UpdateNoteParams {
    pub note: Note,
}

#[derive(Serialize, Deserialize)]
pub struct FindNotesParams {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct NotesInfoParams {
    pub notes: Vec<u64>,
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
}

#[derive(Serialize, Deserialize)]
pub struct NoteAction {
    pub action: String,
    pub version: u8,
    pub params: Params,
}

impl NoteAction {
    pub async fn find_note_ids(
        anki_client: &AnkiClient,
        query: &str,
    ) -> Result<Vec<u64>, AnkiError> {
        let payload = NoteAction {
            action: "findNotes".to_string(),
            version: anki_client.version,
            params: Params::FindNotes(FindNotesParams {
                query: query.to_string(),
            }),
        };

        post_find_note_ids_req(payload, &anki_client.endpoint, &anki_client.client).await
    }

    pub async fn get_notes_infos(
        anki_client: &AnkiClient,
        ids: Vec<u64>,
    ) -> Result<Vec<NotesInfoData>, AnkiError> {
        let payload = NoteAction {
            action: "notesInfo".to_string(),
            version: anki_client.version,
            params: Params::NotesInfo(NotesInfoParams { notes: ids }),
        };

        post_get_notes_infos_req(payload, &anki_client.endpoint, &anki_client.client).await
    }
}

async fn post_get_notes_infos_req(
    payload: NoteAction,
    endpoint: &str,
    client: &Client,
) -> Result<Vec<NotesInfoData>, AnkiError> {
    let res = match client.post(endpoint).json(&payload).send().await {
        Ok(response) => response,
        Err(e) => return Err(AnkiError::RequestError(e.to_string())),
    };

    let body: Result<NotesInfoRes, reqwest::Error> = res.json().await;

    match body {
        Ok(res) => res.into_result(),
        Err(e) => Err(AnkiError::ParseError(e.to_string())),
    }
}

async fn post_find_note_ids_req(
    payload: NoteAction,
    endpoint: &str,
    client: &Client,
) -> Result<Vec<u64>, AnkiError> {
    let res = match client.post(endpoint).json(&payload).send().await {
        Ok(response) => response,
        Err(e) => return Err(AnkiError::RequestError(e.to_string())),
    };

    let body: Result<NumVecRes, reqwest::Error> = res.json().await;

    match body {
        Ok(res) => res.into_result(),
        Err(e) => Err(AnkiError::ParseError(e.to_string())),
    }
}

#![allow(non_snake_case)]
use crate::error::format_error;
use crate::result::NumVecRes;
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


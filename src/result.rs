#![allow(non_snake_case)]

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NumVecRes {
    pub result: Option<Vec<u128>>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FieldData {
    pub value: String,
    pub order: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotesInfoData {
    pub noteId: u128,
    pub modelName: String,
    pub tags: Vec<String>,
    pub fields: IndexMap<String, FieldData>,
}

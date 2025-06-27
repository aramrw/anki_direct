#![allow(non_snake_case)]
use crate::error::AnkiError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub fields: HashMap<String, FieldData>,
}

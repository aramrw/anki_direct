#![allow(non_snake_case)]
use crate::error::AnkiError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// # Example Result
/// ```
/// {
///    "result": [1483959289817, 1483959291695],
///    "error": null
/// }
/// ```
///
/// `NumVecRes` can be returned from the following requests:
/// - FindNotes
#[derive(Serialize, Deserialize, Debug)]
pub struct NumVecRes {
    pub result: Option<Vec<u64>>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FieldData {
   pub value: String,
   pub order: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotesInfoData {
   pub noteId: u64,
   pub modelName: String,
   pub tags: Vec<String>,
   pub fields: HashMap<String, FieldData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotesInfoRes {
   pub result: Option<Vec<NotesInfoData>>,
   pub error: Option<String>,
}

impl NotesInfoRes {
    pub fn into_result(self) -> Result<Vec<NotesInfoData>, AnkiError> {
        match self.error {
            Some(e) => Err(AnkiError::RequestError(e)),
            None => match self.result {
                Some(vec) if vec.is_empty() => Err(AnkiError::NoDataFound),
                Some(vec) => Ok(vec),
                None => Err(AnkiError::NoDataFound),
            },
        }
    }
}

impl NumVecRes {
    pub fn into_result(self) -> Result<Vec<u64>, AnkiError> {
        match self.error {
            Some(e) => Err(AnkiError::RequestError(e)),
            None => match self.result {
                Some(vec) if vec.is_empty() => Err(AnkiError::NoDataFound),
                Some(vec) => Ok(vec),
                None => Err(AnkiError::NoDataFound),
            },
        }
    }
}

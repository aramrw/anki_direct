#![allow(non_snake_case)]
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
    result: Option<Vec<u64>>,
    error: Option<String>,
    pub result: Option<Vec<u64>>,
    pub error: Option<String>,
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

}

impl NumVecRes {
    pub fn into_result(self) -> Result<Option<Vec<u64>>, String> {
        match self.error {
            Some(err) => Err(err),
            None => match self.result {
                Some(vec) if vec.is_empty() => Ok(None),
                Some(vec) => Ok(Some(vec)),
                None => Ok(None),
            },
        }
    }
}

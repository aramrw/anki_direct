#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
//use std::collections::HashMap;

/// `AnkiError` is an enum that represents possible errors that can occur when communicating with Anki.
#[derive(Serialize, Deserialize, Debug)]
pub enum AnkiError {
    /// No data was found for the query.
    NoDataFound,
    /// An error occurred during the request.
    RequestError(String),
    /// An error occurred while parsing the response.
    ParseError(String),
}

impl Error for AnkiError {}

impl Display for AnkiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnkiError::NoDataFound => write!(f, "No data found for query."),
            AnkiError::RequestError(e) => write!(f, "Request error: {e}"),
            AnkiError::ParseError(e) => write!(f, "Parse error: {e}"),
        }
    }
}

/// Formats an error message with a title.
pub fn format_error(title: &str, error: String) -> String {
    format!("{title}: {error}")
}

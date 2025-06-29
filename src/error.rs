#![allow(non_snake_case)]
use std::fmt::Debug;

use serde_json::Value;
use thiserror::Error;

#[cfg(feature = "cache")]
use crate::cache::CacheError;
use crate::{
    generic::GenericRequestBuilderError,
    notes::{MediaBuilderError, NoteBuilderError},
};

pub type AnkiResult<T> = Result<T, AnkiError>;

#[derive(Debug, Error)]
pub enum BuilderErrors {
    #[error("[error/media_builder] missing either one of Url/Path/Data variants for an AnkiMedia type for filename: {0}")]
    MissingMedia(String),
    #[error("{0}")]
    GenericRequest(#[from] GenericRequestBuilderError),
    #[error("{0}")]
    Note(#[from] NoteBuilderError),
    #[error("{0}")]
    Media(#[from] MediaBuilderError),
}

impl From<GenericRequestBuilderError> for AnkiError {
    fn from(value: GenericRequestBuilderError) -> Self {
        Self::Builder(BuilderErrors::GenericRequest(value))
    }
}
impl From<NoteBuilderError> for AnkiError {
    fn from(value: NoteBuilderError) -> Self {
        Self::Builder(BuilderErrors::Note(value))
    }
}
impl From<MediaBuilderError> for AnkiError {
    fn from(value: MediaBuilderError) -> Self {
        Self::Builder(BuilderErrors::Media(value))
    }
}

/// anki error
#[derive(Debug, Error)]
pub enum AnkiError {
    #[error("[error/anki-connect]: {0}")]
    AnkiConnect(String),
    #[error("no data found")]
    NoDataFound,
    #[error("request error: {0}")]
    RequestError(String),
    #[error("[error/reqwest-internal]: {0}")]
    ReqWestError(#[from] reqwest::Error),
    #[error("could not parse str as int: {0}")]
    ParseIntError(String),
    #[error("[error/connection-not-found]: \"{0}\" does not have ankiconnect running\n[help] is anki open?\n[help] confirm the port passed is the same as ankiconnects settings in anki")]
    ConnectionNotFound(String),
    #[error("[error/serde_json(internal)]: {0}")]
    SerdeJson(#[from] serde_json::Error),
    /// this variant is used for debugging only
    #[error("{0}")]
    CustomSerde(#[from] CustomSerdeError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("[error/builder]: {0}")]
    Builder(#[from] BuilderErrors),
    #[cfg(feature = "cache")]
    #[error("[error/cache]: {0}")]
    Cache(#[from] CacheError),
}

#[derive(Debug, Error)]
#[error("(error/debug_serde) -> {{\n{r}\n}} ")]
pub struct CustomSerdeError {
    r: CseReason,
}
#[derive(Debug, Error)]
pub enum CseReason {
    #[error("{0}")]
    ExpectedObject(#[from] ExpectedObject),
}
#[derive(Debug, Error)]
#[error("  [expected]:\n    {exp}\n  [received]:\n    {recv:#?}\n  [help]: {help}")]
pub struct ExpectedObject {
    exp: String,
    recv: Value,
    help: String,
}
impl CustomSerdeError {
    pub fn expected(expected: Option<String>, received: Value, help: Option<String>) -> Self {
        let eo = ExpectedObject {
            exp: unspecified(expected),
            recv: received,
            help: unspecified(help),
        };
        let r = CseReason::ExpectedObject(eo);
        Self { r }
    }
}
fn unspecified(v: Option<String>) -> String {
    match v {
        Some(v) => v,
        None => "<unspecified>".to_string(),
    }
}

impl AnkiError {
    #[inline(always)]
    #[track_caller]
    pub fn pretty_panic(&self) -> ! {
        panic!("<PANIC>\n {self}")
    }
}

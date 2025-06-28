#![allow(non_snake_case)]
use std::fmt::Debug;

use serde_json::Value;
use thiserror::Error;

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

#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::fmt::Display;
//use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub enum AnkiError {
    NoDataFound,
    RequestError(String),
    ParseError(String),
}

pub fn format_error(title: &str, error: String) -> String {
    format!("{}: {}", title, error)
}

#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use std::fmt::Display;
//use std::collections::HashMap;

pub fn format_error(title: &str, error: String) -> String {
    format!("{}: {}", title, error)
}

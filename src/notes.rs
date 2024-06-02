#![allow(non_snake_case)]
use crate::error::format_error;
use crate::result::NumVecRes;
use crate::AnkiClient;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


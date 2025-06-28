use std::fmt::{Debug, Display};

use serde::de::DeserializeOwned;

use crate::str_utils::camel_case_split;

#[derive(Clone, Copy, Debug)]
pub enum AnkiQuery {
    CardState(CardState),
}
impl Display for AnkiQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CardState(state) => Display::fmt(state, f),
        }
    }
}

/// https://docs.ankiweb.net/searching.html#card-state
#[derive(Clone, Copy, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum CardState {
    IsDue,
    IsNew,
    IsLearn,
    IsReview,
    IsSuspended,
}
impl Display for CardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_default_search_query(f, self)
    }
}

/// [Display::fmt] Helper for anki queries that can be guessed from the variant name:
/// ```
/// IsNew -> "is:new"
/// ```
fn fmt_default_search_query(
    f: &mut std::fmt::Formatter<'_>,
    variant: impl Debug,
) -> std::fmt::Result {
    let variant_as_str = format!("{variant:?}");
    let splits = camel_case_split(&variant_as_str);
    if splits.len() > 2 {
        panic!("[unexpected-panic] incorrect variant\n[reason]: you cannot pass a variant that has more than 2 capitals");
    }
    let (left, right) = (splits[0].to_lowercase(), splits[1].to_lowercase());
    write!(f, "{left}:{right}")
}

pub trait AnkiConnectResult<T: DeserializeOwned> {
    fn result(&mut self) -> T;
    fn error(&mut self) -> Option<String>;
}

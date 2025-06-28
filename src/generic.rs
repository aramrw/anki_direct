use std::{borrow::Cow, fmt::Debug};

use derive_builder::Builder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::{
    anki::AnkiConnectResult,
    error::{AnkiError, CustomSerdeError},
    Backend,
};

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct GenericRequest<P: Serialize> {
    #[builder(setter(custom))]
    #[builder(default)]
    action: String,
    #[builder(default)]
    version: u8,
    #[builder(default)]
    params: Option<P>,
}
impl<P: Serialize> GenericRequestBuilder<P> {
    pub fn action(&mut self, action: Cow<str>) -> &mut Self {
        self.action = Some(action.into_owned());
        self
    }
}

#[derive(Serialize, Deserialize)]
pub struct GenericResult<T> {
    pub result: T,
    pub error: Option<String>,
}
impl<T: DeserializeOwned + Default> AnkiConnectResult<T> for GenericResult<T> {
    fn result(&mut self) -> T {
        std::mem::take(&mut self.result)
    }
    fn error(&mut self) -> Option<String> {
        std::mem::take(&mut self.error)
    }
}

impl Backend {
    /// Internal generic request.
    /// `<T>` specifies the `result` field for [GenericResult].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let payload = NoteAction {..};
    /// let res: Result<Vec<isize>> = self.post_generic_request::<Vec<isize>>(payload).await
    /// ```
    pub async fn post_generic_request<T: DeserializeOwned + Debug>(
        &self,
        payload: impl Serialize,
    ) -> Result<T, AnkiError> {
        let (client, endpoint) = (&self.client, &self.endpoint);
        let res = match client.post(endpoint).json(&payload).send().await {
            Ok(response) => response,
            Err(e) => return Err(AnkiError::RequestError(e.to_string())),
        };

        let mut val: Value = res.json().await?;
        if let Some(result_array) = val.get_mut("result").and_then(|r| r.as_array_mut()) {
            result_array.retain(|item| {
                match item.as_object() {
                    // only keep if not empty
                    Some(obj) => !obj.is_empty(),
                    // always keep if not an object
                    None => true,
                }
            });
        }
        let body: GenericResult<T> = serde_json::from_value(val.clone()).map_err(|e| {
            let cse = CustomSerdeError::expected(
                Some(crate::test_utils::display_type::<GenericResult<T>>()),
                val,
                Some(e.to_string()),
            );
            AnkiError::CustomSerde(cse)
        })?;
        if let Some(err) = body.error {
            return Err(AnkiError::AnkiConnect(err));
        }
        Ok(body.result)
    }
}

#![allow(non_snake_case)]
use crate::anki::AnkiQuery;
use crate::error::{AnkiError, AnkiResult, BuilderErrors};
use crate::generic::GenericRequestBuilder;
use crate::result::NotesInfoData;
use crate::{NotesProxy, Number};
use derive_builder::Builder;
use indexmap::IndexMap;
use num_traits::PrimInt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::base64::Base64;
use serde_with::{serde_as, skip_serializing_none};
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DuplicateScope {
    Deck,
    EntireCollection,
}
impl Display for DuplicateScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deck => write!(f, "deck"),
            Self::EntireCollection => write!(f, "entire-collection"),
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteOptions {
    allow_duplicate: bool,
    /// Can be used to specify the scope for which duplicates are checked.
    /// A value of "deck" will only check for duplicates in the target deck;
    /// any other value will check the entire collection.
    duplicate_scope: DuplicateScope,
    duplicate_scope_options: DuplicateScopeOptions,
}
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateScopeOptions {
    /// will specify which deck to use for checking duplicates in.
    /// If None, the target deck will be used.
    deck_name: Option<String>,
    check_children: bool,
    check_all_models: bool,
}

/// A trait for types/collections that can be empty.
///
/// # Usage
/// O(1) garunteed.
/// Less strict than bounding by [Iterator<Item = T>] or [IntoIterator<Item = T>].
trait CollectionExt {
    // 1. Must accurately represent the number of items in the collection.
    // 2. Should only be used for when `len()` is garunteed to be O(1).
    /// Returns the number of items in the collection in O(1) time.
    fn len(&self) -> usize;
    /// Returns `true` if the collection is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T> CollectionExt for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<T> CollectionExt for Option<Vec<T>> {
    fn len(&self) -> usize {
        match self {
            Some(v) => v.len(),
            None => 0,
        }
    }
}
impl CollectionExt for String {
    fn len(&self) -> usize {
        self.len()
    }
}
impl CollectionExt for Option<String> {
    fn len(&self) -> usize {
        match self {
            Some(s) => s.len(),
            None => 0,
        }
    }
}
impl<T> CollectionExt for IndexMap<T, T> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<T> CollectionExt for Option<IndexMap<T, T>> {
    fn len(&self) -> usize {
        match self {
            Some(map) => map.len(),
            None => 0,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Builder, Default)]
#[serde(rename_all = "camelCase")]
#[builder(setter(strip_option))]
#[builder(build_fn(skip))]
pub struct Note {
    deck_name: String,
    model_name: String,
    #[builder(setter(custom))]
    fields: IndexMap<String, String>,
    #[builder(default)]
    options: Option<NoteOptions>,
    #[builder(default)]
    tags: Option<Vec<String>>,
    #[builder(default)]
    #[serde(rename = "audio")]
    audios: Option<Vec<Media>>,
    #[builder(default)]
    #[serde(rename = "video")]
    videos: Option<Vec<Media>>,
    #[builder(default)]
    #[serde(rename = "picture")]
    pictures: Option<Vec<Media>>,
}

impl NoteBuilder {
    fn field(&mut self, field_name: &str, value: &str) -> &mut Self {
        let fields = self.fields.get_or_insert_with(IndexMap::new);
        fields.insert(field_name.into(), value.into());
        self
    }

    /// `audios`, `videos`, and `pictures` fields get mutated into holding [base64] data
    /// aka url or path media get turned into [Vec<u8>] instead of having ankiconnect do the
    /// downloading
    async fn convert_media_fields_to_bytes(
        &mut self,
        client: Option<&Client>,
    ) -> AnkiResult<&mut Self> {
        // Convert media sources to bytes
        if let Some(Some(audios)) = &mut self.audios {
            for audio in audios.iter_mut() {
                audio.data(client).await?;
            }
        }
        if let Some(Some(videos)) = &mut self.videos {
            for video in videos.iter_mut() {
                video.data(client).await?;
            }
        }
        if let Some(Some(pictures)) = &mut self.pictures {
            for picture in pictures.iter_mut() {
                picture.data(client).await?;
            }
        }
        Ok(self)
    }

    // helper to check if a field is not initialized
    // only works for Option<T> where T has a .is_empty() method
    fn is_field_uninitialized<T: CollectionExt>(
        &self,
        field: &Option<T>,
        field_name: &str,
    ) -> AnkiResult<()> {
        if let Some(field) = field {
            if field.is_empty() {
                return Err(AnkiError::Builder(BuilderErrors::Note(
                    NoteBuilderError::UninitializedField(field_name.to_string().leak()),
                )));
            }
        }
        Ok(())
    }

    /// Builds a new note with its media as bytes.
    ///
    /// # Information
    /// [MediaSource::Url] => Downloads verified [url::Url] data & converts to [base64].
    /// [MediaSource::Path] => Calls [tokio::fs::read] on the [PathBuf].
    /// [MediaSource::Data] => Returns the data directly. Expected that the bytes are [base64]
    /// encoded. Should return an error later by checking encoding.
    ///
    /// If you created all media using builder using only `data` field directly,
    /// it will not make any internal requests.
    async fn build(&mut self, client: Option<&Client>) -> AnkiResult<Note> {
        // populates a new Note
        let mut note = Note::default();
        // Ensure that the anki fields is set
        self.is_field_uninitialized::<_>(&self.fields, "fields")?;
        note.fields = self.fields.take().unwrap();

        // Ensure that the deck name is set
        self.is_field_uninitialized::<_>(&self.deck_name, "deck_name")?;
        note.deck_name = self.deck_name.take().unwrap();

        // Ensure that the model name is set
        self.is_field_uninitialized::<_>(&self.model_name, "model_name")?;
        note.model_name = self.model_name.take().unwrap();

        // final step: convert media sources to bytes
        self.convert_media_fields_to_bytes(client).await?;

        note.audios = self.audios.take().flatten();
        note.videos = self.videos.take().flatten();
        note.pictures = self.pictures.take().flatten();
        note.tags = self.tags.take().flatten();
        note.options = self.options.take().flatten();

        Ok(note)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MediaPathError {
    #[error("path does not exist.\n[path]: {0}")]
    DoesntExist(PathBuf),
    #[error("path is neither relative nor absolute.\n[path]: {0}")]
    InvalidVariant(String),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaPath(PathBuf);
impl MediaPath {
    fn from_path(buf: PathBuf) -> Result<Self, MediaPathError> {
        if !buf.exists() {
            return Err(MediaPathError::DoesntExist(buf));
        }
        Ok(Self(buf))
    }
}
impl Deref for MediaPath {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<Path> for MediaPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MediaSourceError {
    #[error("{0}")]
    Path(#[from] MediaPathError),
}
// This enum represents where the media data comes from.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MediaSource {
    /// Raw base64 provided directly.
    Data(Vec<u8>),
    /// A URL to a file that needs to be downloaded.
    Url(String),
    /// A local file path that needs to be read.
    Path(MediaPath),
    /// internal enum used for taking ownership cheaply
    _Empty,
}
impl MediaSource {
    /// Asynchronously resolves the media source into a vector of bytes of [base64]
    ///
    /// This function will either:
    /// - Directly return the data if type is already [MediaSource::Data]
    /// - Download the bytes from a URL if the variant is [MediaSource::Url]
    /// - Read the data from a local file if the variant is [MediaSource::Path]
    pub async fn to_data(&self, client: Option<&Client>) -> Result<Vec<u8>, AnkiError> {
        match self {
            // Variant 1: The data is already here.
            // We just need to clone it to return an owned Vec<u8>.
            MediaSource::Data(data) => Ok(data.clone()),

            // Variant 2: We need to perform an async network request.
            MediaSource::Url(url) => {
                println!("Downloading from URL: {url}");
                let client = match client {
                    Some(ac) => ac,
                    None => &Client::new(),
                };
                let response = client.get(url).send().await?;
                let bytes = response.bytes().await?;
                Ok(bytes.to_vec())
            }

            // Variant 3: We need to perform an async file system read.
            MediaSource::Path(path) => {
                println!("Reading from path: {:?}", path);
                // Use tokio's async file reader. The `?` will convert
                // a `std::io::Error` into our `MediaError::FileIo`.
                let bytes = tokio::fs::read(path).await?;
                Ok(bytes)
            }
            _ => unreachable!(),
        }
    }
}
impl From<&str> for MediaSource {
    fn from(s: &str) -> Self {
        if let Ok(path) = MediaPath::from_path(PathBuf::from(s)) {
            Self::Path(path)
        } else if let Ok(url) = url::Url::parse(s) {
            Self::Url(url.to_string())
        } else {
            let data: Vec<u8> = s.bytes().collect();
            Self::Data(data)
        }
    }
}

#[skip_serializing_none]
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(setter(strip_option))]
pub struct Media {
    filename: String,
    #[serde_as(as = "Base64")]
    #[builder(setter(custom))]
    #[builder(private)]
    #[builder(default)]
    /// required
    data: Vec<u8>,
    #[builder(setter(into))]
    #[builder(default)]
    url: Option<MediaSource>,
    #[builder(setter(into))]
    #[builder(default)]
    path: Option<MediaSource>,
    fields: Vec<String>,
    #[builder(default)]
    skipHash: Option<String>,
}
impl Media {
    // Helper method to create "missing media" error
    fn missing_media_error(&self) -> AnkiResult<&mut Self> {
        Err(AnkiError::Builder(
            crate::error::BuilderErrors::MissingMedia(self.filename.clone()),
        ))
    }
    /// Internal Private Function
    /// Converts media source (url, path, or data) into a vector of bytes.
    ///
    /// This method resolves the media source by:
    /// 1. Using the URL source if available
    /// 2. Using the Path source if available
    /// 3. Using the existing data if available
    /// 4. Returning an error if no source is provided
    ///
    /// # Reason
    /// Avoids AnkiConnect download by preparing data before sending.
    /// Only base64 encoded data is passed to AnkiConnect.
    pub async fn data(&mut self, client: Option<&Client>) -> AnkiResult<&mut Self> {
        // Try to extract data from available sources
        let bytes = match (&mut self.url, &mut self.path, &self.data) {
            (Some(url), _, _) => url.to_data(client).await?,
            (_, Some(path), _) => path.to_data(client).await?,
            (_, _, bytes) => {
                // Data already exists, return early
                if !bytes.is_empty() {
                    return Ok(self);
                }
                return self.missing_media_error();
            }
        };
        self.data = bytes;
        Ok(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GuiEditNoteParams {
    pub note: Number,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateNoteParams {
    pub note: Note,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FindNotesParams {
    pub query: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotesInfoParams {
    pub notes: Vec<Number>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddNotesParams {
    notes: Vec<Note>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Params {
    FindNotes(FindNotesParams),
    NotesInfo(NotesInfoParams),
    GuiEditNote(GuiEditNoteParams),
    AddNote(AddNotesParams),
}

impl NotesProxy {
    pub async fn add_notes(&self, notes: &[Note]) -> AnkiResult<Vec<isize>> {
        let params = json!({
             "notes": notes,
        });
        let payload = GenericRequestBuilder::default()
            .action("addNotes".into())
            .version(self.version)
            .params(Some(params))
            .build()
            .unwrap();
        let res = self
            .post_generic_request::<Option<Vec<isize>>>(payload)
            .await?;
        // safe because generic request will return an err
        // if it's None
        Ok(res.unwrap())
    }
    // fn add_notes() -> AnkiResult<()> {
    //
    // }

    /// Returns note ids from a given query
    ///
    /// # Usage
    ///
    /// ```no_run
    /// let res = Backend
    ///    .find_note_ids(AnkiQuery::CardState(CardState::IsNew))
    ///    .await?;
    /// ```
    ///
    /// # Example Result
    /// ```no_run
    /// {
    ///    "result": [1483959289817, 1483959291695],
    ///    "error": null
    /// }
    /// ```
    pub async fn find_notes(&self, query: AnkiQuery) -> Result<Vec<isize>, AnkiError> {
        let params = Some(Params::FindNotes(FindNotesParams {
            query: query.to_string(),
        }));
        let payload = GenericRequestBuilder::default()
            .action("findNotes".into())
            .version(self.version)
            .params(params)
            .build()
            .unwrap();
        self.post_generic_request::<Vec<isize>>(payload).await
    }

    pub async fn get_notes_infos(
        &self,
        ids: &[impl PrimInt],
    ) -> Result<Vec<NotesInfoData>, AnkiError> {
        let params = Some(Params::NotesInfo(NotesInfoParams {
            notes: Number::from_slice_to_vec(ids),
        }));
        let payload = GenericRequestBuilder::default()
            .action("findNotes".into())
            .version(self.version)
            .params(params)
            .build()
            .unwrap();
        let res = self
            .post_generic_request::<Vec<NotesInfoData>>(payload)
            .await?;
        if res.is_empty() {
            return Err(AnkiError::NoDataFound);
        }
        Ok(res)
    }

    pub async fn gui_edit(&self, id: impl PrimInt) -> Result<(), AnkiError> {
        let params = Some(Params::GuiEditNote(GuiEditNoteParams {
            note: Number::new(id),
        }));
        let payload = GenericRequestBuilder::default()
            .action("guiEditNote".into())
            .version(self.version)
            .params(params)
            .build()
            .unwrap();
        self.post_generic_request::<()>(payload).await?;
        Ok(())
    }

    pub async fn delete_notes_by_ids(&self, ids: &[impl PrimInt]) -> AnkiResult<()> {
        let ids = Number::from_slice_to_vec(ids);
        let params = json!({
            "notes": ids
        });
        let payload = GenericRequestBuilder::default()
            .action("deleteNotes".into())
            .version(self.version)
            .params(Some(params))
            .build()?;
        self.post_generic_request(payload).await
    }
}

#[cfg(test)]
mod note_tests {
    use crate::{
        anki::{AnkiQuery, CardState},
        error::AnkiResult,
        notes::{MediaBuilder, NoteBuilder},
        test_utils::ANKICLIENT,
    };

    #[tokio::test]
    async fn query_note_ids() {
        let res = ANKICLIENT
            .notes()
            .find_notes(AnkiQuery::CardState(CardState::IsNew))
            .await
            .map_err(|e| e.pretty_panic())
            .unwrap();
        assert!(!res.is_empty());
        let res = ANKICLIENT
            .notes()
            .find_notes(AnkiQuery::CardState(CardState::IsLearn))
            .await
            .map_err(|e| e.pretty_panic())
            .unwrap();
        assert!(!res.is_empty())
    }

    #[tokio::test]
    async fn get_notes_infos() {
        ANKICLIENT
            .notes()
            .get_notes_infos(&[12345])
            .await
            .map_err(|e| e.pretty_panic())
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn gui_edit_note() {
        ANKICLIENT
            .notes()
            .gui_edit(1749686091185 as usize)
            .await
            .map_err(|e| e.pretty_panic())
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn add_notes() -> AnkiResult<()> {
        let ac = &ANKICLIENT;
        if let Ok(ids) = ac.notes().find_notes("偽者扱い".into()).await {
            ac.notes().delete_notes_by_ids(&ids).await?;
        }
        let audio = MediaBuilder::create_empty()
            .filename("偽物扱い.mp3".into())
            .fields(vec!["sentenceAudio".into()])
            .url(
                "https://us-southeast-1.linodeobjects.com/immersionkit/media/anime/Hunter%20%C3%97%20Hunter/media/HUNTER%C3%97HUNTER_04_0.20.23.534-0.20.31.042.mp3"
            )
            .build()?;

        let image = MediaBuilder::create_empty()
            .filename("偽者扱い.jpg".into())
            .fields(vec!["picture".into()])
            .url("https://us-southeast-1.linodeobjects.com/immersionkit/media/anime/Hunter%20%C3%97%20Hunter/media/HUNTER%C3%97HUNTER_04_0.20.27.288.jpg")
            .build()?;

        let mut note_builder = NoteBuilder::create_empty();
        let note_futures = vec![note_builder
            .model_name("aramrw".into())
            .deck_name("新文を採れた".into())
            .field("wordDictionaryForm", "偽者扱い")
            .field("reading", "にせものあつかい")
            .field(
                "sentence",
                "私を偽者扱いして 受験者を混乱させ→ 何人か 連れ去ろうとしたのでしょうね。",
            )
            .field("definition", "人を偽の人物として扱いさせる。")
            .tags(vec!["anki-direct".into()])
            .audios(vec![audio])
            .pictures(vec![image])
            .build(Some(ac.reqwest_client()))];

        let notes: Vec<_> = futures::future::join_all(note_futures).await;
        let notes = notes
            .into_iter()
            .map(|res| res.unwrap())
            .collect::<Vec<_>>();

        let new_ids = ac.notes().add_notes(&notes).await?;
        ac.notes().gui_edit(*new_ids.first().unwrap()).await?;
        Ok(())
    }
}

#[cfg(feature = "mock")]
use crate::AnkiApi;
#[cfg(feature = "mock")]
use crate::result::{NotesInfoData, FieldData};
#[cfg(feature = "mock")]
use crate::error::AnkiError;
#[cfg(feature = "mock")]
use async_trait::async_trait;
#[cfg(feature = "mock")]
use std::collections::HashMap;

/// `HardcodedMockAnkiClient` is a mock implementation of the `AnkiApi` trait that returns hardcoded values.
#[cfg(feature = "mock")]
pub struct HardcodedMockAnkiClient;

#[cfg(feature = "mock")]
#[async_trait]
impl AnkiApi for HardcodedMockAnkiClient {
    async fn find_note_ids(&self, _query: &str) -> Result<Vec<u128>, AnkiError> {
        Ok(vec![1678509245459])
    }

    async fn get_notes_infos(&self, ids: Vec<u128>) -> Result<Vec<NotesInfoData>, AnkiError> {
        let mut notes = Vec::new();
        for id in ids {
            if id == 1678509245459 {
                let mut fields = HashMap::new();
                fields.insert("wordDictionaryForm".to_string(), FieldData { value: "<font color=\"#fd4040\">際</font>".to_string(), order: 0 });
                fields.insert("sentence".to_string(), FieldData { value: "この<span style=\"background-color: rgb(19, 19, 19);\"><font color=\"#fd4040\">際</font></span>アンコを<font color=\"#fd6666\">始末</font><font color=\"#9bd2ff\">して</font>は？".to_string(), order: 1 });
                fields.insert("reading".to_string(), FieldData { value: "<span style=\"display:inline;\"><span style=\"display:inline-block;position:relative;padding-right:0.1em;margin-right:0.1em;\"><span style=\"display:inline;\">さ</span><span style=\"border-color:currentColor;display:block;user-select:none;pointer-events:none;position:absolute;top:0.1em;left:0;right:0;height:0;border-top-width:0.1em;border-top-style:solid;right:-0.1em;height:0.4em;border-right-width:0.1em;border-right-style:solid;\"></span></span><span style=\"display:inline-block;position:relative;\"><span style=\"display:inline;\">い</span><span style=\"border-color:currentColor;\"></span></span></span>".to_string(), order: 2 });
                fields.insert("definition".to_string(), FieldData { value: "<ul><li>（①）on the occasion of | circumstances | juncture</li><li>（②）(when preceded by a verb) the moment ... happens | the instant ... occurs</li></ul>".to_string(), order: 3 });
                fields.insert("wordAudio".to_string(), FieldData { value: "[sound:yomichan_audio_さい_際_2023-02-11-04-34-04.mp3]".to_string(), order: 4 });
                fields.insert("sentenceAudio".to_string(), FieldData { value: "".to_string(), order: 5 });
                fields.insert("picture".to_string(), FieldData { value: "<img src=\"mifune-stabs-deidara.jpg\">".to_string(), order: 6 });
                fields.insert("pitchAccent".to_string(), FieldData { value: "<svg xmlns=\"http://www.w3.org/2000/svg\" focusable=\"false\" viewBox=\"0 0 150 100\" style=\"display:inline-block;vertical-align:middle;height:1.5em;\"><path d=\"M25 25 L75 75\" style=\"fill:none;stroke-width:5;stroke:currentColor;\"></path><path d=\"M75 75 L125 75\" style=\"fill:none;stroke-width:5;stroke:currentColor;stroke-dasharray:5 5;\"></path><circle cx=\"25\" cy=\"25\" r=\"15\" style=\"fill:none;stroke-width:5;stroke:currentColor;\"></circle><circle cx=\"25\" cy=\"25\" r=\"5\" style=\"fill:currentColor;\"></circle><circle cx=\"75\" cy=\"75\" r=\"15\" style=\"stroke-width:5;fill:currentColor;stroke:currentColor;\"></circle><path d=\"M0 13 L15 -13 L-15 -13 Z\" transform=\"translate(125,75)\" style=\"fill:none;stroke-width:5;stroke:currentColor;\"></path></svg>".to_string(), order: 7 });
                fields.insert("frequency".to_string(), FieldData { value: "<ul style=\"text-align: left;\"><li>Innocent Corpus: 22206</li></ul>".to_string(), order: 8 });

                notes.push(NotesInfoData {
                    noteId: id,
                    modelName: "aramrw".to_string(),
                    tags: vec!["yomichan".to_string()],
                    fields,
                });
            }
        }
        Ok(notes)
    }

    async fn deck_names(&self) -> Result<Vec<String>, AnkiError> {
        Ok(vec![
            "Default".to_string(),
            "espanol".to_string(),
            "Math".to_string(),
            "Ελληνικά".to_string(),
            "中文汉语普通话".to_string(),
            "新文を採れた".to_string(),
        ])
    }

    async fn gui_edit_note(&self, _id: u128) -> Result<(), AnkiError> {
        Ok(())
    }
}

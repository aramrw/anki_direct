## Rust library for interacting with the [AnkiConnect](https://git.sr.ht/~foosoft/anki-connect) plugin

### Examples

```rust
use anki_direct::prelude::*;
pub static ANKICLIENT: LazyLock<AnkiClient> = LazyLock::new(AnkiClient::default);

#[test]
#[ignore]
    fn add_notes() -> AnkiResult<()> {
        let ac = &ANKICLIENT;
        if let Ok(ids) = ac.notes().find_notes("偽者扱い".into()) {
            ac.notes().delete_notes_by_ids(&ids)?;
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
        let notes: Vec<_> = vec![note_builder
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
            .build(Some(ac.reqwest_client()))?];

        let new_ids = ac.notes().add_notes(&notes)?;
        ac.notes().gui_edit(*new_ids.first().unwrap())?;
        Ok(())
    }
```

# `anki_direct`

A fast, synchronous, strongly-typed Rust client for the [AnkiConnect](https://git.sr.ht/~foosoft/anki-connect) plugin.

`anki_direct` is designed around low-overhead blocking HTTP calls (`reqwest::blocking`), ergonomic builders for notes and media, and optional serializable model/deck caching.

---

## Installation

Add `anki_direct` to your `Cargo.toml`:

```toml
[dependencies]
anki_direct = "0.1.1"

# Or enable the built-in model/deck cache:
anki_direct = { version = "0.1.1", features = ["cache"] }
```

---

## Quick Start

### 1. Connecting to AnkiConnect

```rust
use anki_direct::prelude::*;

fn main() -> AnkiResult<()> {
    // Connects to http://localhost:8765 and verifies the AnkiConnect API version
    let client = AnkiClient::default_latest()?;

    // Or connect to a custom port:
    // let client = AnkiClient::new_port("8765")?;

    // Or initialize lazily without a startup ping (assumes port 8765, API v6):
    // let client = AnkiClient::default();

    println!("AnkiConnect version: {}", client.misc().get_version()?);
    Ok(())
}
```

### 2. Adding Notes with Audio & Images

When you attach media via URL or local file path on [`MediaBuilder`], `NoteBuilder::build` resolves the media into bytes and sends base64 data directly to AnkiConnect—avoiding AnkiConnect-side download hangs or timeouts.

```rust
use anki_direct::prelude::*;
use std::sync::LazyLock;

pub static ANKICLIENT: LazyLock<AnkiClient> = LazyLock::new(AnkiClient::default);

fn add_vocab_note() -> AnkiResult<()> {
    let ac = &ANKICLIENT;

    // Find and remove existing matching notes if desired
    if let Ok(ids) = ac.notes().find_notes("偽者扱い".into()) {
        if !ids.is_empty() {
            ac.notes().delete_notes_by_ids(&ids)?;
        }
    }

    let audio = MediaBuilder::default()
        .filename("偽物扱い.mp3".into())
        .fields(vec!["sentenceAudio".into()])
        .url("https://example.com/audio.mp3")
        .build()?;

    let image = MediaBuilder::default()
        .filename("偽者扱い.jpg".into())
        .fields(vec!["picture".into()])
        .url("https://example.com/image.jpg")
        .build()?;

    let mut note_builder = NoteBuilder::default();
    let note = note_builder
        .model_name("JapaneseVocab".into())
        .deck_name("Mining".into())
        .field("wordDictionaryForm", "偽者扱い")
        .field("reading", "にせものあつかい")
        .field("sentence", "私を偽者扱いして受験者を混乱させようとした。")
        .field("definition", "人を偽の人物として扱うこと。")
        .tags(vec!["anki-direct".into()])
        .audios(vec![audio])
        .pictures(vec![image])
        .build(Some(ac.reqwest_client()))?;

    let new_ids = ac.notes().add_notes(&[note])?;

    // Optionally open the newly created note in the Anki GUI editor:
    if let Some(&first_id) = new_ids.first() {
        ac.notes().gui_edit(first_id)?;
    }

    Ok(())
}
```

### 3. Searching & Inspecting Notes

```rust
use anki_direct::prelude::*;

fn inspect_new_cards(client: &AnkiClient) -> AnkiResult<()> {
    // Search using a typed CardState or a custom Anki search string
    let note_ids = client.notes().find_notes(AnkiQuery::CardState(CardState::IsNew))?;
    // Or: client.notes().find_notes("deck:Mining tag:anki-direct".into())?;

    if !note_ids.is_empty() {
        let infos = client.notes().get_notes_infos(&note_ids)?;
        for info in infos {
            println!("Note {} (model: {}): {:?}", info.noteId, info.modelName, info.tags);
        }
    }
    Ok(())
}
```

### 4. Inspecting Decks & Models (Note Types)

```rust
use anki_direct::prelude::*;

fn list_decks_and_models(client: &AnkiClient) -> AnkiResult<()> {
    // Map of deck_name -> deck_id
    let decks = client.decks().get_all_deck_names_and_ids()?;
    for (name, id) in &decks {
        println!("Deck: {name} (id: {})", **id);
    }

    // Fetch model names + field lists (lightweight)
    let models = client.models().get_all_models_less()?;
    for (name, details) in &models {
        println!("Model: {name}, fields: {:?}", details.fields);
    }

    // Or fetch full model details including card HTML templates and CSS styling:
    let full_models = client.models().get_all_models_full()?;
    for (name, model) in &full_models {
        println!("Model: {name}, is_cloze: {}", model.is_cloze());
    }

    Ok(())
}
```

### 5. Caching & Hydration (`features = ["cache"]`)

When the `cache` feature is enabled, `AnkiClient` includes a serializable [`Cache`](https://docs.rs/anki_direct/latest/anki_direct/cache/struct.Cache.html) for models and decks. Because the live HTTP connection cannot be serialized to disk, deserialized caches start **dehydrated** and can be re-connected via `cache.hydrate(client.modules().clone())`:

```rust
use anki_direct::prelude::*;

#[cfg(feature = "cache")]
fn refresh_cache(client: &mut AnkiClient) -> AnkiResult<()> {
    client.cache_mut().update_all()?;
    println!("Cached {} models and {} decks", client.cache().models().len(), client.cache().decks().len());
    Ok(())
}
```

---

## License

MIT

#[cfg(test)]
mod tests {
    use crate::notes::NoteAction;
    use crate::AnkiClient;

    #[tokio::test]
    async fn test_find_newest_notes() {
        let client = AnkiClient::default();
        let res = NoteAction::find_note_ids(&client, "is:new").await.unwrap();

        // Assert
        assert!(!res.is_empty());
    }

    #[tokio::test]
    async fn fetch_note_info() {
        let client = AnkiClient::default();
        let res = NoteAction::get_notes_infos(&client, vec![1678509245459])
            .await
            .unwrap();
        let word = &res
            .last()
            .unwrap()
            .fields
            .get("wordDictionaryForm")
            .unwrap()
            .value;

        assert!(word.contains("際"));
    }

    #[tokio::test]
    async fn test_deck_names() {
        let client = AnkiClient::default();
        let res = NoteAction::deck_names(&client).await.unwrap();
        assert!(res.contains(&"Default".to_string()));
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_client() {
        use crate::mock::HardcodedMockAnkiClient;
        use crate::AnkiApi;

        let client = HardcodedMockAnkiClient;
        let decks = client.deck_names().await.unwrap();
        assert!(decks.contains(&"Default".to_string()));

        let notes = client.find_note_ids("any").await.unwrap();
        assert_eq!(notes[0], 1678509245459);

        let info = client.get_notes_infos(vec![1678509245459]).await.unwrap();
        assert!(info[0].fields.get("wordDictionaryForm").unwrap().value.contains("際"));
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_mock_api_generated() {
        use crate::MockAnkiApi;
        use crate::AnkiApi;

        let mut mock = MockAnkiApi::new();
        mock.expect_deck_names()
            .times(1)
            .returning(|| Ok(vec!["MockDeck".to_string()]));

        let decks = mock.deck_names().await.unwrap();
        assert_eq!(decks[0], "MockDeck");
    }
}

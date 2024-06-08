#[cfg(test)]
mod tests {
    use crate::notes::NoteAction;
    use crate::AnkiClient;

    #[tokio::test]
    async fn test_find_newest_notes() {
        let client = AnkiClient::default();
        let res = NoteAction::find_note_ids(&client, "is:new").await.unwrap();

        // Assert
        assert_eq!(*res.last().unwrap(), 1717752795958);
    }

    #[tokio::test]
    async fn fetch_note_info() {
        let client = AnkiClient::default();
        let res = NoteAction::get_notes_infos(&client, vec![1717752795958])
            .await
            .unwrap();
        let word = &res
            .last()
            .unwrap()
            .fields
            .get("wordDictionaryForm")
            .unwrap()
            .value;

        assert_eq!(*word, "筒抜け");
    }
}

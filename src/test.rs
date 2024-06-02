#[cfg(test)]
mod tests {
    use crate::notes::NoteAction;
    use crate::AnkiClient;

    #[tokio::test]
    async fn test_find_notes() {
        let client = AnkiClient::default();
        let res = NoteAction::find_note_ids(&client, "is:new").await.unwrap();

        // Assert
        let res = res.unwrap();
        assert_eq!(res, vec![1717316234910]);
    }
}

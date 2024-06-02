
#[cfg(test)]
mod tests {
    use crate::AnkiClient;
    use crate::notes::NoteAction;
    use mockito::Server;

    #[tokio::main]
    async fn main() {
        test_find_notes().await;
    }

    async fn test_find_notes() {
        // Request a new server from the pool
        let mut server = Server::new();

        // Create a mock
        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(r#"{"result": [1483959289817, 1483959291695], "error": null}"#)
            .create();

        let client = AnkiClient::default();
        let res = NoteAction::find_note_ids(&client, "is:new").await;

        // Assert
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Some(vec![1483959289817, 1483959291695]));

        // Verify that the mock was called
        mock.assert();
    }
}


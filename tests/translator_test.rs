#[cfg(test)]
mod tests {
    use movie_metadata_capture::translator::AzureTranslator;

    #[tokio::test]
    async fn test_translate_with_valid_inputs() {
        let translator = AzureTranslator::new(
            String::from("https://api.cognitive.microsofttranslator.com/translate?api-version=3.0"),
            String::from("ACCESS_KEY"),
            Some("japaneast".to_string()),
        );

        let result = translator.translate("Hello", "en", "fr").await;

        assert!(result.is_some());
        assert_eq!(result.unwrap(), String::from("Bonjour"));
    }

    #[tokio::test]
    async fn test_translate_with_invalid_account_key() {
        let translator = AzureTranslator::new(
            String::from("https://api.cognitive.microsofttranslator.com"),
            String::from("INVALID_ACCESS_KEY"), // Use an invalid access key to simulate an error
            None,
        );

        let result = translator.translate("Hello", "en", "fr").await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_translate_with_invalid_from_lang() {
        let translator = AzureTranslator::new(
            String::from("https://api.cognitive.microsofttranslator.com"),
            String::from("ACCESS_KEY"),
            None,
        );

        let result = translator.translate("Hello", "xxx", "fr").await; // Use an invalid from_lang to simulate an error

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_translate_with_invalid_to_lang() {
        let translator = AzureTranslator::new(
            String::from("https://api.cognitive.microsofttranslator.com"),
            String::from("ACCESS_KEY"),
            None,
        );

        let result = translator.translate("Hello", "en", "xxx").await; // Use an invalid to_lang to simulate an error

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_translate_with_empty_text() {
        let translator = AzureTranslator::new(
            String::from("https://api.cognitive.microsofttranslator.com"),
            String::from("ACCESS_KEY"),
            None,
        );

        let result = translator.translate("", "en", "fr").await; // Use an empty text to simulate an error

        assert!(result.is_none());
    }
}

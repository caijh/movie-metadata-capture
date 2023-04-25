#[cfg(test)]
mod tests {
    use movie_metadata_capture::number_parser::{get_number, get_number_by_dict};

    #[test]
    fn test_get_number_by_dict() {
        // Test getting number by dictionary lookup
        assert_eq!(get_number_by_dict("filename_1"), Some("1".to_string()));
        assert_eq!(get_number_by_dict("subtitle_2"), Some("2".to_string()));
        assert_eq!(get_number_by_dict("test"), None);
    }

    #[test]
    fn test_get_number_default_cases() {
        // Test default case where no specific pattern is detected
        let number = get_number("");
        assert_eq!(number, Some("".to_string()));
        assert_eq!(get_number("test"), None);
    }

    #[test]
    fn test_get_number_by_fanza_cid() {
        // Test matching FANZA CID pattern
        assert_eq!(
            get_number("filename_ABC.11.11.11_sample.mp4"),
            Some("ABC.11.11.11".to_string())
        );
        assert_eq!(get_number("filename_AB.1.1.1_sample.mp4"), None);
    }

    #[test]
    fn test_get_number_by_chinese_pattern() {
        // Test matching chinese subtitles pattern
        assert_eq!(get_number("filename_字幕组_1.mp4"), Some("1".to_string()));
        assert_eq!(get_number("filename_字幕_2.mp4"), None);
    }

    #[test]
    fn test_get_number_by_dash_pattern() {
        // Test matching filenames with dash pattern
        assert_eq!(get_number("filename-1.mp4"), Some("1".to_string()));
        assert_eq!(
            get_number("filename-s08-e11-sample.mp4"),
            Some("s08".to_string())
        );
        assert_eq!(get_number("filename_s01_e01.mp4"), None);
    }

    #[test]
    fn test_get_number_by_bracket_pattern() {
        // Test matching filenames with bracket pattern
        assert_eq!(
            get_number("[filename]_sample_1.mp4"),
            Some("sample".to_string())
        );
        assert_eq!(
            get_number("[filename_2].mp4"),
            Some("filename_2".to_string())
        );
        assert_eq!(get_number("filename.mp4"), None);
    }
}

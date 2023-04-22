use movie_metadata_capture::files::file_exit_and_not_empty;
use std::io::Write;

#[test]
fn test_file_exit_and_not_empty() {
    // create a temporary file to use for testing
    let mut temp_file = std::fs::File::create("temp.txt").unwrap();
    temp_file.write_all(b"test").unwrap();

    // test the function with an existing non-empty file
    let path = std::path::Path::new("temp.txt");
    assert_eq!(file_exit_and_not_empty(&path), true);

    // test the function with a non-existing file
    let path = std::path::Path::new("non_existing.txt");
    assert_eq!(file_exit_and_not_empty(&path), false);

    // test the function with an existing empty file
    let empty_file = std::fs::File::create("empty.txt").unwrap();
    let path = std::path::Path::new("empty.txt");
    assert_eq!(file_exit_and_not_empty(&path), false);

    // clean up temporary files
    std::fs::remove_file("temp.txt").unwrap();
    std::fs::remove_file("empty.txt").unwrap();
}

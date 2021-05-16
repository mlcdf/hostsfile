use ho;

#[test]
fn test_open_existing_file() {
    let _ = ho::hosts::HostsFile::open("tests/fixtures/hostsfile".to_string()).unwrap();
}

#[test]
fn test_fail_open_missing_file() {
    let result = ho::hosts::HostsFile::open("tests/fixtures/do-not-exists".to_string());
    assert_eq!(result.is_err(), true)
}

#[test]
fn test_missing_end_tag_file() {
    let result = ho::hosts::HostsFile::open("tests/fixtures/missing-end-hostsfile".to_string());
    assert_eq!(result.is_err(), true)
}

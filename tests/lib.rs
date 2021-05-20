use std::fs;
use std::str;

use hostsfile;

#[test]
fn test_open_existing_file() {
    hostsfile::File::open("tests/fixtures/hostsfile".to_string()).unwrap();
}

#[test]
fn test_fail_open_missing_file() {
    hostsfile::File::open("tests/fixtures/do-not-exists".to_string()).unwrap_err();
}

#[test]
fn test_missing_end_tag_file() {
    hostsfile::File::open("tests/fixtures/missing-end-hostsfile".to_string()).unwrap_err();
}

#[test]
fn test_file_with_tags() {
    let mut file = hostsfile::File::open("tests/fixtures/hostsfile-with-tags".to_string()).unwrap();
    let mut writer = Vec::new();

    let mut entries = Vec::<hostsfile::Entry>::new();
    entries.push("127.0.0.2 pi.mlcdf.fr".parse::<hostsfile::Entry>().unwrap());

    file.write(&entries, &mut writer).unwrap();
    let got = str::from_utf8(&writer).unwrap();

    let expected = fs::read_to_string("tests/fixtures/hostsfile-with-tags.result".to_string())
        .expect("Something went wrong reading the file");

    assert_eq!(expected, got);
}

use ho::config;
use ho::hosts;

#[test]
fn test_open_existing_file() {
    hosts::File::open("tests/fixtures/hostsfile".to_string()).unwrap();
}

#[test]
#[should_panic]
fn test_fail_open_missing_file() {
    hosts::File::open("tests/fixtures/do-not-exists".to_string()).unwrap();
}

#[test]
#[should_panic]
fn test_missing_end_tag_file() {
    hosts::File::open("tests/fixtures/missing-end-hostsfile".to_string()).unwrap();
}

#[test]
fn test_file_with_tags() {
    let mut hostsfile =
        hosts::File::open("tests/fixtures/hostsfile-with-tags".to_string()).unwrap();

    let mut writer = Vec::new();
    let mut cfg = config::Hosts::new();

    let ip: std::net::IpAddr = "127.0.0.2".parse().unwrap();
    cfg.insert(ip, vec!["pi.mlcdf.fr".to_string()]);

    hostsfile.write(&cfg, &mut writer).unwrap();

    let got = std::str::from_utf8(&writer).unwrap();

    let expected = std::fs::read_to_string("tests/fixtures/hostsfile-with-tags.result".to_string())
        .expect("Something went wrong reading the file");

    // println!("{:}", got);
    assert_eq!(got, expected);
}

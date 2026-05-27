use std::{fs, io::Write};

use mc_launcher_core::{
    io::archive::extract_zip_safely,
    net::download::{should_skip_existing, Checksum, DownloadTask},
};

#[test]
fn skips_existing_file_when_sha1_matches() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("hello.txt");
    fs::write(&file, b"minecraft").unwrap();

    let task = DownloadTask {
        url: "https://example.invalid/hello.txt".to_string(),
        destination: file,
        checksum: Some(Checksum::Sha1(
            "624c22a8c8f8c93f18fe5ecd4713100c8d754507".to_string(),
        )),
        label: "hello".to_string(),
    };

    assert!(should_skip_existing(&task).unwrap());
}

#[test]
fn rejects_zip_entry_that_escapes_destination() {
    let dir = tempfile::tempdir().unwrap();
    let zip_path = dir.path().join("bad.zip");
    let file = fs::File::create(&zip_path).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    writer
        .start_file("../escape.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    writer.write_all(b"bad").unwrap();
    writer.finish().unwrap();

    let err = extract_zip_safely(&zip_path, dir.path().join("out")).unwrap_err();
    assert!(err.to_string().contains("unsafe path"));
}

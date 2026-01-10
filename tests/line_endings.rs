use flate2::read::GzDecoder;
use std::io::Read;
use tempfile::NamedTempFile;
use tise::{LoadedSave, SaveFormat, TiValue};

fn assert_all_lf_are_crlf(bytes: &[u8]) {
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            assert!(i > 0 && bytes[i - 1] == b'\r', "found bare LF at {i}");
        }
    }
}

#[test]
fn modified_json5_preserves_crlf() {
    let mut tmp = NamedTempFile::new().expect("tempfile");
    let input = b"{\r\n    a: 1,\r\n}\r\n";
    std::io::Write::write_all(&mut tmp, input).expect("write");

    let mut save = LoadedSave::load_path(tmp.path()).expect("load");
    *save.root.get_mut("a").expect("a exists") = TiValue::String("changed".to_string());
    save.mark_dirty();

    let bytes = save
        .save_bytes_for_format(SaveFormat::Json5)
        .expect("bytes");

    assert_all_lf_are_crlf(&bytes);
}

#[test]
fn modified_json5_preserves_lf() {
    let mut tmp = NamedTempFile::new().expect("tempfile");
    let input = b"{\n    a: 1,\n}\n";
    std::io::Write::write_all(&mut tmp, input).expect("write");

    let mut save = LoadedSave::load_path(tmp.path()).expect("load");
    *save.root.get_mut("a").expect("a exists") = TiValue::String("changed".to_string());
    save.mark_dirty();

    let bytes = save
        .save_bytes_for_format(SaveFormat::Json5)
        .expect("bytes");

    assert!(
        !bytes.contains(&b'\r'),
        "expected no CR characters in LF output"
    );
}

#[test]
fn modified_gz_preserves_crlf_inside() {
    let tmp = NamedTempFile::new().expect("tempfile");
    // Any name ending in .gz triggers gzip format detection.
    let gz_path = tmp.path().with_file_name("line_endings_test.json.gz");

    // Write a gzip file containing CRLF JSON5.
    {
        let bytes = b"{\r\n    a: 1,\r\n}\r\n";
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        std::io::Write::write_all(&mut encoder, bytes).expect("gz write");
        let gz_bytes = encoder.finish().expect("gz finish");
        std::fs::write(&gz_path, gz_bytes).expect("write gz");
    }

    let mut save = LoadedSave::load_path(&gz_path).expect("load");
    *save.root.get_mut("a").expect("a exists") = TiValue::String("changed".to_string());
    save.mark_dirty();

    let gz_bytes = save
        .save_bytes_for_format(SaveFormat::GzipJson5)
        .expect("bytes");

    let mut decoder = GzDecoder::new(&gz_bytes[..]);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).expect("decompress");

    assert_all_lf_are_crlf(&out);
}

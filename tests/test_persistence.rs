use simstring_rust::{
    CharacterNgrams, Cosine, Database, Dice, ExactMatch, FeatureExtractor, HashDb, Jaccard,
    Measure, Overlap, PersistenceError, Searcher, WordNgrams,
};
use std::{fs, io, sync::Arc};

fn assert_searches_equal(left: &HashDb, right: &HashDb) {
    for measure in [
        &Cosine as &dyn Measure,
        &Dice,
        &Jaccard,
        &Overlap,
        &ExactMatch,
    ] {
        let expected = Searcher::new(left, measure);
        let actual = Searcher::new(right, measure);
        for query in ["", "aaaa", "café", "東京🍕", "foo,bar", "unseen 🦀"] {
            for alpha in [0.3, 0.6, 0.9, 1.0] {
                assert_eq!(
                    expected.search(query, alpha).unwrap(),
                    actual.search(query, alpha).unwrap()
                );
                assert_eq!(
                    expected.ranked_search(query, alpha).unwrap(),
                    actual.ranked_search(query, alpha).unwrap()
                );
            }
        }
    }
}

#[test]
fn roundtrips_builtins_and_remains_mutable() {
    let extractors: Vec<Arc<dyn FeatureExtractor>> = vec![
        Arc::new(CharacterNgrams::new(0, "$")),
        Arc::new(CharacterNgrams::new(1, "")),
        Arc::new(CharacterNgrams::new(2, "$")),
        Arc::new(CharacterNgrams::new(3, "🍕!")),
        Arc::new(CharacterNgrams::new(100, "")),
        Arc::new(WordNgrams::new(0, " ", "#")),
        Arc::new(WordNgrams::new(2, ",", "##")),
        Arc::new(WordNgrams::new(100, " ", "$")),
    ];
    let directory = tempfile::tempdir().unwrap();
    for extractor in extractors {
        for populated in [false, true] {
            let mut original = HashDb::new(Arc::clone(&extractor));
            if populated {
                for text in ["", "aaaa", "aaaa", "café", "東京🍕", "foo,bar", "foo,baz"] {
                    original.insert(text.into());
                }
            }
            original
                .save_to_file(directory.path(), "original.bin")
                .unwrap();
            let mut restored = HashDb::load_from_file(directory.path(), "original.bin").unwrap();
            assert_eq!(original.strings, restored.strings);
            assert_eq!(
                original.feature_extractor().persistence_config(),
                restored.feature_extractor().persistence_config()
            );
            assert_searches_equal(&original, &restored);
            restored
                .save_to_file(directory.path(), "restored.bin")
                .unwrap();
            assert_eq!(
                fs::read(directory.path().join("original.bin")).unwrap(),
                fs::read(directory.path().join("restored.bin")).unwrap()
            );
            for db in [&mut original, &mut restored] {
                db.insert("unseen 🦀".into());
            }
            assert_searches_equal(&original, &restored);
            for db in [&mut original, &mut restored] {
                db.clear();
                db.insert("café".into());
            }
            assert_searches_equal(&original, &restored);
        }
    }
}

#[test]
fn accepts_strings_larger_than_io_buffers() {
    let mut db = HashDb::new(Arc::new(CharacterNgrams::new(0, "")));
    db.insert("東京🍕".repeat(20_000));
    let directory = tempfile::tempdir().unwrap();
    db.save_to_file(directory.path(), "long.bin").unwrap();
    let loaded = HashDb::load_from_file(directory.path(), "long.bin").unwrap();
    assert_eq!(db.strings, loaded.strings);
    assert_eq!(loaded.feature_sizes(), [0]);
}

#[test]
fn custom_extractors_fail_without_touching_the_destination() {
    struct Custom;
    impl FeatureExtractor for Custom {
        fn visit_features(&self, _: &str, visit: &mut dyn FnMut(&str)) {
            visit("custom1");
        }
    }
    let db = HashDb::new(Arc::new(Custom));
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("existing.bin");
    fs::write(&target, b"previous contents").unwrap();
    assert!(matches!(
        db.save_to_file(directory.path(), "existing.bin"),
        Err(PersistenceError::UnsupportedExtractor)
    ));
    assert_eq!(fs::read(target).unwrap(), b"previous contents");
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn preserves_io_errors_and_validates_filenames() {
    let directory = tempfile::tempdir().unwrap();
    let db = HashDb::new(Arc::new(CharacterNgrams::default()));
    assert!(
        matches!(HashDb::load_from_file(directory.path(), "missing.bin"), Err(PersistenceError::Io(error)) if error.kind() == io::ErrorKind::NotFound)
    );
    assert!(matches!(
        db.save_to_file(directory.path().join("missing"), "db.bin"),
        Err(PersistenceError::Io(_))
    ));
    for name in [
        "",
        ".",
        "..",
        "nested/db.bin",
        "./db.bin",
        "/db.bin",
        "db.bin/",
    ] {
        assert!(
            matches!(
                db.save_to_file(directory.path(), name),
                Err(PersistenceError::InvalidFileName)
            ),
            "{name}"
        );
        assert!(
            matches!(
                HashDb::load_from_file(directory.path(), name),
                Err(PersistenceError::InvalidFileName)
            ),
            "{name}"
        );
    }
    fs::create_dir(directory.path().join("occupied")).unwrap();
    fs::write(directory.path().join("occupied/keep"), b"keep").unwrap();
    assert!(matches!(
        db.save_to_file(directory.path(), "occupied"),
        Err(PersistenceError::Io(_))
    ));
    assert_eq!(
        fs::read(directory.path().join("occupied/keep")).unwrap(),
        b"keep"
    );
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn rejects_corrupted_truncated_and_future_files() {
    let directory = tempfile::tempdir().unwrap();
    let mut db = HashDb::new(Arc::new(CharacterNgrams::default()));
    db.insert("hello".into());
    db.save_to_file(directory.path(), "db.bin").unwrap();
    let path = directory.path().join("db.bin");
    let original = fs::read(&path).unwrap();
    for length in [0, 7, 23, original.len() - 1] {
        fs::write(&path, &original[..length]).unwrap();
        assert!(matches!(
            HashDb::load_from_file(directory.path(), "db.bin"),
            Err(PersistenceError::InvalidSnapshot(_))
        ));
    }
    for offset in [0, 12, 20, original.len() - 1] {
        let mut bytes = original.clone();
        bytes[offset] ^= 1;
        fs::write(&path, bytes).unwrap();
        assert!(matches!(
            HashDb::load_from_file(directory.path(), "db.bin"),
            Err(PersistenceError::InvalidSnapshot(_))
        ));
    }
    let mut bytes = original.clone();
    bytes[8..12].copy_from_slice(&99u32.to_le_bytes());
    fs::write(&path, bytes).unwrap();
    assert!(matches!(
        HashDb::load_from_file(directory.path(), "db.bin"),
        Err(PersistenceError::UnsupportedVersion(99))
    ));
    let mut bytes = original;
    bytes.push(0);
    fs::write(path, bytes).unwrap();
    assert!(HashDb::load_from_file(directory.path(), "db.bin").is_err());
}

#[test]
fn v1_fixtures_are_readable_and_reproducible() {
    let fixtures: [(&str, Arc<dyn FeatureExtractor>); 2] = [
        ("character", Arc::new(CharacterNgrams::new(3, "🍕!"))),
        ("word", Arc::new(WordNgrams::new(2, ",", "#"))),
    ];
    let directory = tempfile::tempdir().unwrap();
    for (name, extractor) in fixtures {
        let hex = fs::read_to_string(format!(
            "{}/tests/fixtures/persistence_v1_{name}.hex",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let bytes: Vec<u8> = hex
            .split_whitespace()
            .map(|byte| u8::from_str_radix(byte, 16).unwrap())
            .collect();
        fs::write(directory.path().join("fixture.bin"), &bytes).unwrap();
        let loaded = HashDb::load_from_file(directory.path(), "fixture.bin").unwrap();
        let mut expected = HashDb::new(extractor);
        for text in ["", "aaaa", "aaaa", "東京🍕", "foo,bar", "café"] {
            expected.insert(text.into());
        }
        assert_eq!(expected.strings, loaded.strings);
        assert_searches_equal(&expected, &loaded);
        expected
            .save_to_file(directory.path(), "generated.bin")
            .unwrap();
        assert_eq!(
            fs::read(directory.path().join("generated.bin")).unwrap(),
            bytes
        );
    }
}

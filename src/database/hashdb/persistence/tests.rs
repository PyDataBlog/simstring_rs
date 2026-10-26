use super::*;
use crate::{Cosine, Database, Searcher};

fn database() -> HashDb {
    let mut db = HashDb::new(Arc::new(CharacterNgrams::new(2, "$")));
    for text in ["hello", "help", "hello", "東京🍕", ""] {
        db.insert(text.into());
    }
    db
}

fn snapshot(db: &HashDb) -> SnapshotV1 {
    let reference = SnapshotRef {
        db,
        extractor: db.feature_extractor.persistence_config().unwrap().into(),
    };
    postcard::from_bytes(&postcard::to_stdvec(&reference).unwrap()).unwrap()
}

fn write_snapshot(path: &Path, snapshot: &SnapshotV1) {
    let payload = postcard::to_stdvec(snapshot).unwrap();
    let mut bytes = header(payload.len() as u64, crc32fast::hash(&payload)).to_vec();
    bytes.extend(payload);
    std::fs::write(path, bytes).unwrap();
}

#[test]
fn rejects_inconsistent_indexes_with_valid_checksums() {
    let db = database();
    let directory = tempfile::tempdir().unwrap();
    let cases: &[fn(&mut SnapshotV1)] = &[
        |s| {
            s.strings.pop();
        },
        |s| {
            s.dictionary.push(s.dictionary[0].clone());
        },
        |s| {
            s.dictionary[1] = s.dictionary[0].clone();
        },
        |s| {
            s.string_features[0][0] = u32::MAX;
        },
        |s| {
            s.string_features[0].reverse();
        },
        |s| {
            s.postings[0][0].1[0] = u64::MAX;
        },
        |s| {
            s.postings
                .iter_mut()
                .flatten()
                .find(|(_, ids)| ids.len() > 1)
                .unwrap()
                .1
                .reverse();
        },
        |s| {
            s.postings[0][0].0 += 1;
        },
        |s| {
            s.postings[0][0].1.clear();
        },
        |s| {
            s.postings[0].clear();
        },
        |s| {
            s.feature_sizes.clear();
        },
        |s| {
            s.feature_sizes.push(u64::MAX);
        },
    ];
    for (index, mutate) in cases.iter().enumerate() {
        let mut value = snapshot(&db);
        mutate(&mut value);
        write_snapshot(&directory.path().join("invalid.bin"), &value);
        assert!(
            HashDb::load_from_file(directory.path(), "invalid.bin").is_err(),
            "case {index}"
        );
    }
}

#[test]
fn preserves_exact_state_and_dictionary_ids() {
    let db = database();
    let restored = snapshot(&db).restore().unwrap();
    assert_eq!(db.strings, restored.strings);
    assert_eq!(db.string_features, restored.string_features);
    assert_eq!(db.postings, restored.postings);
    assert_eq!(db.feature_sizes, restored.feature_sizes);
    assert!(db.interner.iter().eq(restored.interner.iter()));
}

#[test]
fn restores_saved_features_instead_of_reextracting_text() {
    let mut value = snapshot(&database());
    value.strings[0] = "stored label".into();
    let restored = value.restore().unwrap();
    let searcher = Searcher::new(&restored, Cosine);
    let results = searcher.search("hello", 1.0).unwrap();
    assert_eq!(results, ["hello", "stored label"]);
}

#[test]
fn validates_before_replacing_a_file() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("db.bin");
    let mut db = database();
    db.save_to_file(directory.path(), "db.bin").unwrap();
    let original = std::fs::read(&target).unwrap();
    db.strings.pop();
    assert!(matches!(
        db.save_to_file(directory.path(), "db.bin"),
        Err(PersistenceError::InvalidSnapshot(_))
    ));
    assert_eq!(std::fs::read(target).unwrap(), original);
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
}

#[test]
fn retains_os_errors_when_postcard_discards_them() {
    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::from_raw_os_error(28))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = PayloadWriter::new(FailingWriter);
    assert!(postcard::to_io(&42u32, &mut writer).is_err());
    assert_eq!(writer.error.unwrap().raw_os_error(), Some(28));
}

#[test]
fn rejects_trailing_payload_and_invalid_encoding() {
    let directory = tempfile::tempdir().unwrap();
    let db = database();
    let mut payload = postcard::to_stdvec(&snapshot(&db)).unwrap();
    payload.push(0);
    for payload in [payload, vec![0xff; 32]] {
        let mut bytes = header(payload.len() as u64, crc32fast::hash(&payload)).to_vec();
        bytes.extend(payload);
        std::fs::write(directory.path().join("db.bin"), bytes).unwrap();
        assert!(HashDb::load_from_file(directory.path(), "db.bin").is_err());
    }
    assert_eq!(db.total_strings(), 5);
}

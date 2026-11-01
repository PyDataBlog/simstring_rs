use super::HashDb;
use crate::extractors::{CharacterNgrams, ExtractorConfig, FeatureExtractor, WordNgrams};
use lasso::{Capacity, Key, Rodeo, Spur};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::TryReserveError;
use std::fs::File;
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

const MAGIC: &[u8; 8] = b"SIMSTRDB";
const VERSION: u32 = 1;
const HEADER_LEN: usize = 24;

type FeatureIdV1 = u32;

/// Errors returned when saving or loading a database snapshot.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum PersistenceError {
    /// The snapshot could not be read or written.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// `file_name` was not a single file name.
    #[error("file_name must be a single filename")]
    InvalidFileName,
    /// The database's feature extractor does not support snapshots.
    #[error("this feature extractor does not support snapshots")]
    UnsupportedExtractor,
    /// The snapshot uses an unsupported format version.
    #[error("unsupported snapshot version: {0}")]
    UnsupportedVersion(u32),
    /// The snapshot contents are malformed.
    #[error("invalid snapshot: {0}")]
    InvalidSnapshot(&'static str),
    /// The snapshot payload could not be encoded or decoded.
    #[error("invalid snapshot encoding: {0}")]
    Codec(#[from] postcard::Error),
    /// Memory for the restored snapshot could not be allocated.
    #[error("cannot allocate snapshot: {0}")]
    Allocation(#[from] TryReserveError),
    /// The feature dictionary could not be restored.
    #[error("cannot restore feature dictionary: {0}")]
    Interner(#[from] lasso::LassoError),
}

type Result<T> = std::result::Result<T, PersistenceError>;

// Field order and enum tags are part of the v1 format, including extractor semantics.
#[derive(Serialize, Deserialize)]
enum ExtractorV1 {
    Character {
        n: u64,
        padding: String,
    },
    Word {
        n: u64,
        splitter: String,
        padder: String,
    },
}

impl From<ExtractorConfig> for ExtractorV1 {
    fn from(config: ExtractorConfig) -> Self {
        match config {
            ExtractorConfig::Character { n, padding } => Self::Character {
                n: n as u64,
                padding,
            },
            ExtractorConfig::Word {
                n,
                splitter,
                padder,
            } => Self::Word {
                n: n as u64,
                splitter,
                padder,
            },
        }
    }
}

impl ExtractorV1 {
    fn restore(self) -> Result<Arc<dyn FeatureExtractor>> {
        Ok(match self {
            Self::Character { n, padding } => {
                Arc::new(CharacterNgrams::from_padding(to_usize(n)?, padding))
            }
            Self::Word {
                n,
                splitter,
                padder,
            } => Arc::new(WordNgrams::new(to_usize(n)?, &splitter, &padder)),
        })
    }
}

#[derive(Serialize, Deserialize)]
struct SnapshotV1 {
    extractor: ExtractorV1,
    strings: Vec<String>,
    dictionary: Vec<String>,
    string_features: Vec<Vec<FeatureIdV1>>,
    postings: Vec<Vec<(u64, Vec<u64>)>>,
    feature_sizes: Vec<u64>,
}

struct Sequence<I>(I);

impl<I> Serialize for Sequence<I>
where
    I: Iterator + Clone,
    I::Item: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.clone())
    }
}

struct SnapshotRef<'a> {
    db: &'a HashDb,
    extractor: ExtractorV1,
}

impl Serialize for SnapshotRef<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let db = self.db;
        let mut state = serializer.serialize_struct("SnapshotV1", 6)?;
        state.serialize_field("extractor", &self.extractor)?;
        state.serialize_field("strings", &db.strings)?;
        state.serialize_field(
            "dictionary",
            &Sequence((0..db.interner.len()).map(|id| {
                db.interner
                    .resolve(&Spur::try_from_usize(id).expect("existing interner key"))
            })),
        )?;
        state.serialize_field(
            "string_features",
            &Sequence(
                db.string_features
                    .iter()
                    .map(|features| Sequence(features.iter().copied().map(encode_feature_id))),
            ),
        )?;
        state.serialize_field(
            "postings",
            &Sequence(db.postings.iter().map(|buckets| {
                Sequence(
                    buckets.iter().map(|(size, ids)| {
                        (*size as u64, Sequence(ids.iter().map(|&id| id as u64)))
                    }),
                )
            })),
        )?;
        state.serialize_field(
            "feature_sizes",
            &Sequence(db.feature_sizes.iter().map(|&size| size as u64)),
        )?;
        state.end()
    }
}

impl SnapshotV1 {
    fn restore(self) -> Result<HashDb> {
        validate_dictionary_len(self.dictionary.len() as u64)?;
        let mut interner: Rodeo =
            Rodeo::with_capacity(Capacity::for_strings(self.dictionary.len()));
        for (index, text) in self.dictionary.into_iter().enumerate() {
            let key = interner.try_get_or_intern(text)?;
            check(key.into_usize() == index, "duplicate dictionary entry")?;
        }
        let db = HashDb {
            feature_extractor: self.extractor.restore()?,
            strings: self.strings,
            string_features: self
                .string_features
                .into_iter()
                .map(|features| {
                    features
                        .into_iter()
                        .map(decode_feature_id)
                        .collect::<Result<_>>()
                })
                .collect::<Result<_>>()?,
            postings: self
                .postings
                .into_iter()
                .map(|buckets| {
                    buckets
                        .into_iter()
                        .map(|(size, ids)| {
                            Ok((
                                to_usize(size)?,
                                ids.into_iter().map(to_usize).collect::<Result<_>>()?,
                            ))
                        })
                        .collect::<Result<_>>()
                })
                .collect::<Result<_>>()?,
            feature_sizes: self
                .feature_sizes
                .into_iter()
                .map(to_usize)
                .collect::<Result<_>>()?,
            interner,
        };
        db.validate_snapshot()?;
        Ok(db)
    }
}

impl HashDb {
    /// Saves the complete index and built-in extractor configuration to `path/file_name`.
    /// The directory must exist. An existing file is replaced atomically after a
    /// successful write. Returns an error for unsupported custom extractors.
    pub fn save_to_file(&self, path: impl AsRef<Path>, file_name: impl AsRef<Path>) -> Result<()> {
        let destination = destination(path.as_ref(), file_name.as_ref())?;
        let extractor = self
            .feature_extractor
            .persistence_config()
            .ok_or(PersistenceError::UnsupportedExtractor)?;
        self.validate_snapshot()?;
        let snapshot = SnapshotRef {
            db: self,
            extractor: extractor.into(),
        };
        let mut temporary = tempfile::NamedTempFile::new_in(path)?;
        temporary.write_all(&[0; HEADER_LEN])?;
        let mut payload = PayloadWriter::new(temporary.as_file_mut());
        let result = {
            let mut writer = BufWriter::new(&mut payload);
            postcard::to_io(&snapshot, &mut writer)
                .map(|_| ())
                .map_err(PersistenceError::Codec)
                .and_then(|()| writer.flush().map_err(PersistenceError::Io))
        };
        if let Some(error) = payload.error {
            return Err(PersistenceError::Io(error));
        }
        result?;
        let header = header(payload.bytes, payload.checksum.finalize());
        temporary.seek(SeekFrom::Start(0))?;
        temporary.write_all(&header)?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(destination)
            .map_err(|error| PersistenceError::Io(error.error))?;
        Ok(())
    }

    /// Loads a mutable database from a versioned snapshot without extracting features
    /// or rebuilding postings. Reads the payload into memory and validates it before
    /// returning. The extractor is restored from the snapshot.
    pub fn load_from_file(path: impl AsRef<Path>, file_name: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(destination(path.as_ref(), file_name.as_ref())?)?;
        let file_len = file.metadata()?.len();
        let mut header = [0; HEADER_LEN];
        read_exact(&mut file, &mut header)?;
        check(&header[..8] == MAGIC, "wrong magic bytes")?;
        let version = u32::from_le_bytes(header[8..12].try_into().unwrap());
        if version != VERSION {
            return Err(PersistenceError::UnsupportedVersion(version));
        }
        let length = u64::from_le_bytes(header[12..20].try_into().unwrap());
        check(
            length.checked_add(HEADER_LEN as u64) == Some(file_len),
            "file length mismatch",
        )?;
        let mut payload = Vec::new();
        let length = to_usize(length)?;
        payload.try_reserve_exact(length)?;
        payload.resize(length, 0);
        read_exact(&mut file, &mut payload)?;
        check(file.read(&mut [0])? == 0, "trailing file data")?;
        let checksum = u32::from_le_bytes(header[20..24].try_into().unwrap());
        check(crc32fast::hash(&payload) == checksum, "checksum mismatch")?;
        let (snapshot, remaining): (SnapshotV1, _) = postcard::take_from_bytes(&payload)?;
        check(remaining.is_empty(), "trailing payload data")?;
        drop(payload);
        snapshot.restore()
    }

    fn validate_snapshot(&self) -> Result<()> {
        validate_dictionary_len(self.interner.len() as u64)?;
        check(
            self.strings.len() == self.string_features.len(),
            "string/feature count mismatch",
        )?;
        check(
            self.interner.len() == self.postings.len(),
            "dictionary/posting count mismatch",
        )?;
        check(
            self.feature_sizes.windows(2).all(|pair| pair[0] < pair[1]),
            "unsorted feature sizes",
        )?;
        let mut populated = vec![false; self.feature_sizes.len()];
        let mut cursors = vec![0; self.strings.len()];
        for features in &self.string_features {
            check(
                features.windows(2).all(|pair| pair[0] < pair[1]),
                "unsorted string features",
            )?;
            check(
                features
                    .last()
                    .is_none_or(|id| id.into_usize() < self.interner.len()),
                "feature ID out of bounds",
            )?;
            let bucket = self
                .feature_sizes
                .binary_search(&features.len())
                .map_err(|_| PersistenceError::InvalidSnapshot("missing size bucket"))?;
            populated[bucket] = true;
        }
        check(
            populated.into_iter().all(|used| used),
            "unpopulated size bucket",
        )?;
        for (feature, buckets) in self.postings.iter().enumerate() {
            check(
                buckets.windows(2).all(|pair| pair[0].0 < pair[1].0),
                "unsorted posting sizes",
            )?;
            for (size, ids) in buckets {
                check(!ids.is_empty(), "empty posting list")?;
                check(
                    ids.windows(2).all(|pair| pair[0] < pair[1]),
                    "unsorted posting IDs",
                )?;
                check(
                    ids.last().is_none_or(|&id| id < self.strings.len()),
                    "string ID out of bounds",
                )?;
                for &id in ids {
                    let features = &self.string_features[id];
                    check(*size == features.len(), "posting size mismatch")?;
                    check(
                        features
                            .get(cursors[id])
                            .is_some_and(|key| key.into_usize() == feature),
                        "posting membership mismatch",
                    )?;
                    cursors[id] += 1;
                }
            }
        }
        check(
            cursors
                .iter()
                .zip(&self.string_features)
                .all(|(&seen, features)| seen == features.len()),
            "missing posting membership",
        )
    }
}

fn check(valid: bool, message: &'static str) -> Result<()> {
    if valid {
        Ok(())
    } else {
        Err(PersistenceError::InvalidSnapshot(message))
    }
}

fn encode_feature_id(key: Spur) -> FeatureIdV1 {
    FeatureIdV1::try_from(key.into_usize()).expect("validated v1 feature ID")
}

fn decode_feature_id(id: FeatureIdV1) -> Result<Spur> {
    usize::try_from(id)
        .ok()
        .and_then(Spur::try_from_usize)
        .ok_or(PersistenceError::InvalidSnapshot("feature ID overflow"))
}

fn validate_dictionary_len(len: u64) -> Result<()> {
    check(
        len.checked_sub(1).is_none_or(|last| {
            FeatureIdV1::try_from(last).is_ok_and(|id| decode_feature_id(id).is_ok())
        }),
        "too many dictionary entries",
    )
}

fn to_usize(value: u64) -> Result<usize> {
    usize::try_from(value)
        .ok()
        .filter(|&n| n <= isize::MAX as usize)
        .ok_or(PersistenceError::InvalidSnapshot(
            "integer exceeds platform capacity",
        ))
}

fn destination(path: &Path, name: &Path) -> Result<PathBuf> {
    let mut components = name.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(value)), None) if value == name.as_os_str() => Ok(path.join(name)),
        _ => Err(PersistenceError::InvalidFileName),
    }
}

fn header(length: u64, checksum: u32) -> [u8; HEADER_LEN] {
    let mut header = [0; HEADER_LEN];
    header[..8].copy_from_slice(MAGIC);
    header[8..12].copy_from_slice(&VERSION.to_le_bytes());
    header[12..20].copy_from_slice(&length.to_le_bytes());
    header[20..24].copy_from_slice(&checksum.to_le_bytes());
    header
}

fn read_exact(reader: &mut impl Read, bytes: &mut [u8]) -> Result<()> {
    reader.read_exact(bytes).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            PersistenceError::InvalidSnapshot("truncated file")
        } else {
            PersistenceError::Io(error)
        }
    })
}

struct PayloadWriter<W> {
    inner: W,
    bytes: u64,
    checksum: crc32fast::Hasher,
    error: Option<io::Error>,
}

impl<W> PayloadWriter<W> {
    fn new(inner: W) -> Self {
        Self {
            inner,
            bytes: 0,
            checksum: crc32fast::Hasher::new(),
            error: None,
        }
    }
}

impl<W: Write> Write for PayloadWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self.inner.write(bytes) {
            Ok(count) => {
                self.bytes += count as u64;
                self.checksum.update(&bytes[..count]);
                Ok(count)
            }
            Err(error) => {
                let kind = error.kind();
                self.error = Some(error);
                Err(kind.into())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests;
